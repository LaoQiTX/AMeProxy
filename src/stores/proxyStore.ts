import { defineStore } from 'pinia';
import type { Connection, Rule, Log, ProxyGroup, Subscription, Proxy, TrafficData } from '../types';
import {
  desktopInvoke, getProxies, changeProxy, testProxy, getProviders, closeAllConnections,
  is_proxy_running, startCore, startProxy, stopProxy, getProxyStatus, isDesktop, getControllerConfig, webSocketUrl, getTunStatus,
} from '../services/proxy';
import { TelemetrySocket, runLimited } from '../services/telemetry';

let sockets: TelemetrySocket[] = [];
let pollTimer: ReturnType<typeof setTimeout> | null = null;
let generation = 0;
let subscriptionGeneration = 0;
const message = (error: unknown) => error instanceof Error ? error.message : String(error);

export const useProxyStore = defineStore('proxy', {
  state: () => ({
    currentTab: 'dashboard',
    isConnected: false,
    systemProxyEnabled: false,
    recoveryPending: false,
    previewMode: false,
    isBusy: false,
    subscriptionBusy: false,
    tunBusy: false,
    initialized: false,
    error: '',
    selectedKernel: 'Mihomo',
    kernels: ['Mihomo'],
    isTesting: false,
    tunMode: false,
    mixedPort: 7890,
    allowLan: false,
    connections: [] as Connection[],
    rules: [] as Rule[],
    logs: [] as Log[],
    proxyGroups: [] as ProxyGroup[],
    subscriptions: [] as Subscription[],
    activeSubscription: '',
    proxies: [] as Proxy[],
    trafficData: { up: '0 B/s', down: '0 B/s' } as TrafficData,
    trafficTotal: { up: 0, down: 0 },
    uptime: 0,
  }),
  actions: {
    async initialize() {
      if (this.initialized) return;
      this.initialized = true;
      this.previewMode = !isDesktop();
      if (this.previewMode) return;
      this.isBusy = true;
      try {
        await startCore();
        await this.activateSession();
        await this.refreshProxyStatus();
      } catch (error) {
        this.resetSession();
        this.error = message(error);
        await this.refreshProxyStatus().catch(() => {});
      } finally {
        this.isBusy = false;
      }
      // Saved subscriptions remain editable even when startup fails.
      await this.fetchProviders().catch(() => {});
      await this.fetchRules().catch(() => {});
    },

    async activateSession() {
      this.stopPolling();
      const config = await getControllerConfig();
      this.isConnected = true;
      this.trafficTotal = { up: 0, down: 0 };
      let previous = new Map<string, { bytes: number; time: number }>();
      sockets = [
        new TelemetrySocket(webSocketUrl(config, '/traffic'), data => {
          this.trafficData = { up: this.formatBytes(data.up) + '/s', down: this.formatBytes(data.down) + '/s' };
        }),
        new TelemetrySocket(webSocketUrl(config, '/logs?level=info'), data => {
          this.logs.unshift({ time: new Date().toLocaleTimeString(), level: String(data.type).toUpperCase(), msg: String(data.payload) });
          if (this.logs.length > 200) this.logs.length = 200;
        }),
        new TelemetrySocket(webSocketUrl(config, '/connections'), data => {
          this.trafficTotal = { up: data.uploadTotal || 0, down: data.downloadTotal || 0 };
          const now = Date.now();
          const next = new Map<string, { bytes: number; time: number }>();
          this.connections = (data.connections || []).map((c: any) => {
            const last = previous.get(c.id);
            const bytes = Number(c.download) || 0;
            const speed = last && now > last.time ? Math.max(0, bytes - last.bytes) * 1000 / (now - last.time) : 0;
            next.set(c.id, { bytes, time: now });
            return {
              id: c.id, host: c.metadata?.host || c.metadata?.destinationIP || '',
              ip: c.metadata?.destinationIP || '',
              process: (c.metadata?.processPath || '').split(/[\\/]/).pop(),
              rule: c.rule, group: c.chains?.[0] || '', speed: this.formatBytes(speed) + '/s',
              time: new Date(c.start).toLocaleTimeString(),
            };
          });
          previous = next;
        }),
      ];
      await this.fetchProxies().catch(error => { this.error = message(error); });
      this.tunMode = await getTunStatus().catch(() => false);
      this.startPolling();
    },

    async toggleConnection() {
      if (this.isBusy || this.subscriptionBusy) return;
      this.isBusy = true;
      this.error = '';
      try {
        if (this.systemProxyEnabled || this.recoveryPending || this.tunMode) {
          await stopProxy();
          this.resetSession();
          this.systemProxyEnabled = false;
          this.recoveryPending = false;
        } else {
          await startProxy();
          await this.refreshProxyStatus();
          if (!this.isConnected) await this.activateSession();
          await this.fetchProviders();
          await this.fetchRules();
        }
      } catch (error) {
        this.error = message(error);
        if (!await is_proxy_running().catch(() => false)) this.resetSession();
        await this.refreshProxyStatus().catch(() => {});
      } finally { this.isBusy = false; }
    },

    async refreshProxyStatus() {
      const status = await getProxyStatus();
      this.systemProxyEnabled = status.systemProxyEnabled;
      this.recoveryPending = status.recoveryPending;
      if (status.error) this.error = status.error;
      return status;
    },

    resetSession() {
      this.isConnected = false;
      this.stopPolling();
      this.uptime = 0;
      this.tunMode = false;
      this.trafficData = { up: '0 B/s', down: '0 B/s' };
      this.trafficTotal = { up: 0, down: 0 };
      this.connections = [];
      this.proxyGroups = [];
      this.proxies = [];
      this.subscriptions.forEach(sub => { sub.status = 'saved'; sub.count = 0; });
    },

    startPolling() {
      if (pollTimer) clearTimeout(pollTimer);
      const current = generation;
      const poll = async () => {
        if (current !== generation || !this.isConnected) return;
        try {
          const status = await this.refreshProxyStatus();
          if (current !== generation) return;
          if (!status.kernelRunning) {
            this.resetSession();
            this.error = status.error || '内核已退出，请检查日志后重新开启代理';
            return;
          }
          await Promise.all([this.fetchProxies(), this.fetchUptime()]);
        } catch (error) {
          if (current === generation) this.error = message(error);
        } finally {
          if (current === generation && this.isConnected) {
            pollTimer = setTimeout(poll, document.hidden ? 15000 : 5000);
          }
        }
      };
      pollTimer = setTimeout(poll, 1000);
    },

    stopPolling() {
      generation++;
      if (pollTimer) clearTimeout(pollTimer);
      pollTimer = null;
      sockets.forEach(socket => socket.stop());
      sockets = [];
    },

    async fetchProviders() {
      const revision = subscriptionGeneration;
      const config = await desktopInvoke<Record<string, any>>('get_config');
      if (revision !== subscriptionGeneration) return;
      this.mixedPort = Number(config['mixed-port']) || 0;
      this.allowLan = config['allow-lan'] === true;
      const saved = config['proxy-providers'] || {};
      const runtime = this.isConnected ? await getProviders().catch(() => null) : null;
      if (revision !== subscriptionGeneration) return;
      this.activeSubscription = config['active-subscription'] || '';
      this.subscriptions = Object.entries(saved).filter(([, p]: [string, any]) => p?.url).map(([name, value]) => {
        const active = name === this.activeSubscription;
        const provider = active ? runtime?.providers?.[name] : undefined;
        const count = provider?.proxies?.length || 0;
        return {
          name, url: (value as any).url, count,
          updateTime: provider?.updatedAt ? new Date(provider.updatedAt).toLocaleString() : '尚未更新',
          status: count > 0 ? 'ready' : active && this.isConnected ? 'loading' : 'saved',
        } as Subscription;
      });
    },

    async fetchRules() {
      const rules = await desktopInvoke<string[]>('get_rule_config');
      this.rules = rules.map(raw => {
        const [type, payload, strategy] = raw.split(',');
        return { raw, type, payload: type === 'MATCH' ? '' : payload, strategy: type === 'MATCH' ? payload : strategy };
      });
    },

    async fetchProxies() {
      const current = generation;
      const revision = subscriptionGeneration;
      const data = await getProxies();
      if (current !== generation || revision !== subscriptionGeneration || !this.isConnected) return;
      const groups: ProxyGroup[] = [];
      const nodes: Proxy[] = [];
      for (const [name, proxy] of Object.entries(data.proxies || {})) {
        if (Array.isArray(proxy.all)) {
          groups.push({ name, type: proxy.type, options: proxy.all, selected: proxy.now || '' });
        } else if (!['Direct', 'Reject', 'RejectDrop', 'Pass', 'Compatible'].includes(proxy.type)) {
          nodes.push({ name, type: proxy.type, delay: proxy.history?.at(-1)?.delay || 0, region: name.slice(0, 2) });
        }
      }
      this.proxyGroups = groups;
      this.proxies = nodes;
    },

    async switchProxy(groupName: string, proxyName: string) {
      if (this.subscriptionBusy) throw new Error('正在切换订阅，请稍后选择节点');
      try {
        await changeProxy(groupName, proxyName);
        const group = this.proxyGroups.find(group => group.name === groupName);
        if (group) group.selected = proxyName;
      } catch (error) {
        this.error = message(error);
        throw error;
      }
    },

    async testLatency() {
      if (this.isTesting || this.subscriptionBusy || !this.isConnected) return;
      this.isTesting = true;
      const current = generation;
      const revision = subscriptionGeneration;
      try {
        await runLimited<string>(this.proxies.map(p => p.name), 6, async name => {
          if (current !== generation || revision !== subscriptionGeneration) return;
          const delay = await testProxy(name).catch(() => -1);
          if (current !== generation || revision !== subscriptionGeneration) return;
          const proxy = this.proxies.find(p => p.name === name);
          if (proxy) proxy.delay = delay;
        });
      } finally { this.isTesting = false; }
    },

    async saveSubscription(command: string, args: Record<string, unknown>, name?: string, start = false) {
      if (this.subscriptionBusy || this.isBusy) throw new Error('请等待当前操作完成');
      this.subscriptionBusy = true;
      subscriptionGeneration++;
      this.error = '';
      try {
        await desktopInvoke<void>(command, args);
        await this.fetchProviders();
        if (start && name === this.activeSubscription && !this.isConnected) {
          await startCore();
          await this.activateSession();
        }
        await this.fetchProviders();
        if (this.isConnected) {
          await this.fetchProxies();
          if (name && name === this.activeSubscription) {
            const deadline = Date.now() + 6000;
            while (!this.subscriptions.some(s => s.name === name && s.status === 'ready') && Date.now() < deadline) {
              await new Promise(resolve => setTimeout(resolve, 500));
              await this.fetchProviders();
            }
            const subscription = this.subscriptions.find(s => s.name === name);
            if (subscription && subscription.status !== 'ready') {
              subscription.status = 'error';
              this.error = '订阅已保存，但内核尚未返回节点。请检查链接与订阅格式后重试。';
            }
          }
        }
      } catch (error) {
        this.error = message(error);
        throw error;
      } finally { this.subscriptionBusy = false; }
    },

    addSubscription(name: string, url: string) {
      return this.saveSubscription('add_proxy_provider', { name, url }, name);
    },
    importSubscription(name: string, url: string) {
      return this.saveSubscription('add_proxy_provider', { name, url }, name, true);
    },
    updateSubscription(oldName: string, newName: string, url: string) {
      return this.saveSubscription('update_proxy_provider', { oldName, newName, url }, newName);
    },
    removeSubscription(name: string) {
      return this.saveSubscription('remove_proxy_provider', { name });
    },
    switchSubscription(name: string) {
      if (name === this.activeSubscription) return Promise.resolve();
      return this.saveSubscription('set_active_subscription', { name }, name, true);
    },

    async toggleTunMode() {
      if (this.tunBusy || !this.isConnected) return;
      this.tunBusy = true;
      try {
        await desktopInvoke<void>('toggle_tun', { enabled: !this.tunMode });
        this.tunMode = await getTunStatus();
      } catch (error) { this.error = message(error); }
      finally { this.tunBusy = false; }
    },
    setKernel(kernel: string) { this.selectedKernel = kernel; },
    setCurrentTab(tab: string) { this.currentTab = tab; },
    async fetchUptime() {
      const current = generation;
      const uptime = await desktopInvoke<number>('get_uptime');
      if (current === generation) this.uptime = uptime;
    },
    async closeAllConnections() {
      try { await closeAllConnections(); this.connections = []; }
      catch (error) { this.error = message(error); }
    },
    formatBytes(bytes: number) {
      if (!Number.isFinite(bytes) || bytes <= 0) return '0 B';
      const index = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), 4);
      return (bytes / Math.pow(1024, index)).toFixed(2) + ' ' + ['B', 'KB', 'MB', 'GB', 'TB'][index];
    },
  },
});
