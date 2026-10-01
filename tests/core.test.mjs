import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
import ts from 'typescript';
import * as pinia from 'pinia';

function load(file, dependencies = {}, globals = {}) {
  const { outputText } = ts.transpileModule(fs.readFileSync(new URL('../' + file, import.meta.url), 'utf8'), {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
  });
  const exports = {};
  vm.runInNewContext(outputText, { exports, require: name => {
    assert.ok(name in dependencies, 'Unexpected import: ' + name);
    return dependencies[name];
  }, URL, console, setTimeout, clearTimeout, ...globals });
  return exports;
}

test('desktop services use registered command names and preserve arguments', async () => {
  const calls = [];
  let desktop = true;
  const service = load('src/services/proxy.ts', {
    '@tauri-apps/api/core': { isTauri: () => desktop, invoke: async (...args) => { calls.push(args); return 42; } },
  });
  await service.getProxies();
  await service.getProviders();
  await service.changeProxy('默认', '日本/%?#');
  assert.equal(await service.testProxy('日本/%?#'), 42);
  await service.closeAllConnections();
  assert.deepEqual(JSON.parse(JSON.stringify(calls)), [
    ['get_proxies', null], ['get_providers', null],
    ['change_proxy', { group: '默认', proxy: '日本/%?#' }],
    ['test_proxy', { proxy: '日本/%?#' }], ['close_all_connections', null],
  ]);
  const url = new URL(service.webSocketUrl({ wsUrl: 'ws://127.0.0.1:1234', secret: 'a&b?#' }, '/logs?level=info'));
  assert.equal(url.searchParams.get('level'), 'info');
  assert.equal(url.searchParams.get('token'), 'a&b?#');
  desktop = false;
  await assert.rejects(service.startCore(), /桌面应用/);
  assert.equal(calls.length, 5);
});

test('stopping telemetry cancels retries and ignores stale frames', () => {
  const instances = [], timers = new Map();
  let next = 0;
  class Socket {
    constructor() { instances.push(this); }
    close() { this.onclose?.(); }
  }
  const { TelemetrySocket } = load('src/services/telemetry.ts', {}, {
    WebSocket: Socket,
    setTimeout: fn => { timers.set(++next, fn); return next; },
    clearTimeout: id => timers.delete(id),
  });
  const received = [];
  const telemetry = new TelemetrySocket('ws://example.invalid', data => received.push(data));
  instances[0].onmessage({ data: '1' });
  instances[0].onerror();
  assert.equal(timers.size, 1);
  telemetry.stop();
  assert.equal(timers.size, 0);
  instances[0].onmessage({ data: '2' });
  instances[0].onclose();
  assert.deepEqual(received, [1]);
  assert.equal(timers.size, 0);
});

test('latency jobs respect the concurrency bound', async () => {
  const { runLimited } = load('src/services/telemetry.ts');
  let active = 0, maximum = 0, finished = 0;
  await runLimited(Array.from({ length: 20 }, (_, i) => i), 3, async () => {
    active++;
    maximum = Math.max(maximum, active);
    await new Promise(resolve => setImmediate(resolve));
    active--; finished++;
  });
  assert.equal(maximum, 3);
  assert.equal(finished, 20);
});

test('saved subscriptions without runtime nodes are never reported as loaded', async () => {
  pinia.setActivePinia(pinia.createPinia());
  const service = {
    desktopInvoke: async () => ({ 'proxy-providers': { demo: { url: 'https://example.invalid' } }, 'active-subscription': 'demo', 'mixed-port': 1234, 'allow-lan': false }),
    getProviders: async () => ({ providers: {} }),
  };
  const { useProxyStore } = load('src/stores/proxyStore.ts', {
    pinia, '../services/proxy': service, '../services/telemetry': {},
  });
  const store = useProxyStore();
  store.isConnected = true;
  await store.fetchProviders();
  assert.equal(store.subscriptions[0].status, 'loading');
  assert.equal(store.subscriptions[0].count, 0);
  assert.equal(store.subscriptions[0].updateTime, '尚未更新');
  store.isConnected = false;
  await store.fetchProviders();
  assert.equal(store.subscriptions[0].status, 'saved');
});

test('startup failures remain disconnected and surface an actionable error', async () => {
  pinia.setActivePinia(pinia.createPinia());
  const { useProxyStore } = load('src/stores/proxyStore.ts', {
    pinia,
    '../services/proxy': {
      isDesktop: () => true,
      startCore: async () => { throw new Error('控制端口被占用'); },
      desktopInvoke: async command => command === 'get_rule_config' ? [] : { 'proxy-providers': {} },
    },
    '../services/telemetry': {},
  });
  const store = useProxyStore();
  await store.initialize();
  assert.equal(store.isConnected, false);
  assert.equal(store.isBusy, false);
  assert.match(store.error, /控制端口被占用/);
});

function connectionStore(service) {
  pinia.setActivePinia(pinia.createPinia());
  const { useProxyStore } = load('src/stores/proxyStore.ts', {
    pinia,
    '../services/proxy': {
      isDesktop: () => true,
      desktopInvoke: async command => command === 'get_rule_config' ? [] : { 'proxy-providers': {} },
      getProviders: async () => ({ providers: {} }),
      ...service,
    },
    '../services/telemetry': {},
  });
  return useProxyStore();
}

test('main switch enables system proxy even when kernel is already running, then restores on disable', async () => {
  const calls = [];
  let enabled = false;
  const store = connectionStore({
    startProxy: async () => { calls.push('enable'); enabled = true; },
    stopProxy: async () => { calls.push('restore'); enabled = false; },
    getProxyStatus: async () => ({ kernelRunning: true, systemProxyEnabled: enabled, recoveryPending: enabled, error: '' }),
  });
  store.isConnected = true;
  await store.toggleConnection();
  assert.equal(store.isConnected, true);
  assert.equal(store.systemProxyEnabled, true);
  assert.deepEqual(calls, ['enable']);
  await store.toggleConnection();
  assert.equal(store.isConnected, false);
  assert.equal(store.systemProxyEnabled, false);
  assert.equal(store.recoveryPending, false);
  assert.deepEqual(calls, ['enable', 'restore']);
});

test('failed restore keeps actual proxy state visible and allows retry', async () => {
  const store = connectionStore({
    stopProxy: async () => { throw new Error('系统代理恢复失败'); },
    is_proxy_running: async () => true,
    getProxyStatus: async () => ({ kernelRunning: true, systemProxyEnabled: true, recoveryPending: true, error: '' }),
  });
  store.isConnected = true;
  store.systemProxyEnabled = true;
  store.recoveryPending = true;
  await store.toggleConnection();
  assert.equal(store.isConnected, true);
  assert.equal(store.systemProxyEnabled, true);
  assert.equal(store.recoveryPending, true);
  assert.equal(store.isBusy, false);
  assert.match(store.error, /恢复失败/);
});

test('external proxy changes update status without claiming the kernel stopped', async () => {
  const store = connectionStore({
    getProxyStatus: async () => ({ kernelRunning: true, systemProxyEnabled: false, recoveryPending: true, error: '' }),
  });
  store.isConnected = true;
  store.systemProxyEnabled = true;
  await store.refreshProxyStatus();
  assert.equal(store.isConnected, true);
  assert.equal(store.systemProxyEnabled, false);
});

test('browser preview never starts the kernel or shows a native startup error', async () => {
  const store = connectionStore({ isDesktop: () => false, startCore: () => assert.fail('must not call native commands') });
  await store.initialize();
  assert.equal(store.previewMode, true);
  assert.equal(store.isConnected, false);
  assert.equal(store.systemProxyEnabled, false);
  assert.equal(store.error, '');
});

test('inactive subscriptions remain saved even if stale runtime data contains their nodes', async () => {
  const store = connectionStore({
    desktopInvoke: async () => ({ 'active-subscription': 'A', 'proxy-providers': { A: { url: 'https://a.invalid' }, B: { url: 'https://b.invalid' } } }),
    getProviders: async () => ({ providers: { A: { proxies: [{ name: 'a' }] }, B: { proxies: [{ name: 'stale' }] } } }),
  });
  store.isConnected = true;
  await store.fetchProviders();
  assert.equal(store.activeSubscription, 'A');
  assert.equal(store.subscriptions[0].status, 'ready');
  assert.equal(store.subscriptions[1].status, 'saved');
  assert.equal(store.subscriptions[1].count, 0);
});

test('switch subscription refreshes selected provider and nodes; failed switch preserves selection', async () => {
  let active = 'A';
  const calls = [];
  const store = connectionStore({
    desktopInvoke: async (command, args) => {
      if (command === 'set_active_subscription') {
        calls.push(args.name);
        if (args.name === 'broken') throw new Error('配置校验失败');
        active = args.name;
      }
      return { 'active-subscription': active, 'proxy-providers': { A: { url: 'https://a.invalid' }, B: { url: 'https://b.invalid' } } };
    },
    getProviders: async () => ({ providers: { [active]: { proxies: [{ name: active + '-node' }] } } }),
    getProxies: async () => ({ proxies: { 默认: { type: 'Selector', all: [active + '-node'], now: active + '-node' } } }),
  });
  store.isConnected = true;
  await store.fetchProviders();
  await store.switchSubscription('B');
  assert.equal(store.activeSubscription, 'B');
  assert.equal(store.proxyGroups[0].selected, 'B-node');
  assert.equal(store.subscriptions[0].status, 'saved');
  await store.switchSubscription('B');
  assert.deepEqual(calls, ['B']);
  await assert.rejects(store.switchSubscription('broken'), /配置校验失败/);
  assert.equal(store.activeSubscription, 'B');
  assert.equal(store.subscriptionBusy, false);
});

test('importing an inactive subscription does not start the kernel or wait for its nodes', async () => {
  let added = false;
  const store = connectionStore({
    desktopInvoke: async command => {
      if (command === 'add_proxy_provider') added = true;
      return { 'active-subscription': 'A', 'proxy-providers': { A: { url: 'https://a.invalid' }, ...(added ? { B: { url: 'https://b.invalid' } } : {}) } };
    },
    startCore: async () => assert.fail('inactive import must not start the kernel'),
  });
  await store.importSubscription('B', 'https://b.invalid');
  assert.equal(store.activeSubscription, 'A');
  assert.equal(store.subscriptions[1].status, 'saved');
  assert.equal(store.error, '');
});
