<script setup lang="ts">
import { computed, ref } from 'vue';
import { Activity, ArrowDown, ArrowRight, ArrowUp, CheckCircle2, CircleHelp, Clock3, Cpu, Gauge, Globe2, Layers, LoaderCircle, RefreshCw, SearchCheck, Settings2, Shield, TriangleAlert, WifiOff } from 'lucide-vue-next';
import { useProxyStore } from '../../stores/proxyStore';
import type { ProxyMode, DiagnosticCheck } from '../../services/proxy';
import ProxyToggle from '../common/ProxyToggle.vue';

const store = useProxyStore();
const switching = ref(false);
const diagnosticsOpen = ref(false);
const modes: { value: ProxyMode; label: string }[] = [
  { value: 'rule', label: '规则' }, { value: 'global', label: '全局' }, { value: 'direct', label: '直连' },
];
const checkLabels: Record<DiagnosticCheck['key'], string> = {
  kernel: '代理核心', config: '配置文件', systemProxy: '系统代理',
  port: '本地端口', dns: 'DNS 查询', target: '目标连通性',
};
const checkKeys = Object.keys(checkLabels) as DiagnosticCheck['key'][];
const activeSubscription = computed(() => store.subscriptions.find(sub => sub.name === store.activeSubscription));
const currentGroup = computed(() => store.proxyGroups.find(group => group.name === (store.proxyMode === 'global' ? 'GLOBAL' : '默认'))
  || store.proxyGroups.find(group => group.type === 'Selector' && group.name !== 'GLOBAL')
  || store.proxyGroups.find(group => group.name === 'GLOBAL') || store.proxyGroups[0]);
const selected = computed(() => currentGroup.value?.selected || '');
const currentNode = computed(() => store.proxies.find(node => node.name === selected.value));
const nodeIsDirect = computed(() => ['DIRECT', '直连'].includes(selected.value));
const nodeIsGroup = computed(() => store.proxyGroups.some(group => group.name === selected.value));
const coreStatus = computed(() => store.isBusy ? '处理中' : store.isConnected ? '运行中' : '未运行');
const proxyStatus = computed(() => store.systemProxyEnabled ? '已接管' : store.recoveryPending ? '待恢复' : '未接管');
const connectionSummary = computed(() => store.isBusy ? '正在更新连接状态' : store.systemProxyEnabled
  ? '系统代理已开启' : store.tunMode ? 'TUN 已开启' : store.isConnected ? '核心已就绪' : '代理未开启');
const uptime = computed(() => {
  if (!store.isConnected) return '--:--:--';
  const seconds = store.uptime;
  return [Math.floor(seconds / 3600), Math.floor(seconds % 3600 / 60), seconds % 60]
    .map(value => String(value).padStart(2, '0')).join(':');
});
const nodeDelay = computed(() => currentNode.value?.delay && currentNode.value.delay > 0 ? `${currentNode.value.delay} ms`
  : currentNode.value?.delay && currentNode.value.delay < 0 ? '测速失败' : '尚未测速');
const diagnosticRows = computed(() => checkKeys.map(key => ({
  key, label: checkLabels[key],
  ...(store.diagnosticChecks.find(item => item.key === key) || { state: 'untested', detail: '尚未检测' }),
})));

async function selectNode(event: Event) {
  const group = currentGroup.value;
  if (!group || switching.value) return;
  const target = event.target as HTMLSelectElement;
  switching.value = true;
  try { await store.switchProxy(group.name, target.value); }
  catch { target.value = group.selected; }
  finally { switching.value = false; }
}
async function selectSubscription(event: Event) {
  const target = event.target as HTMLSelectElement;
  try { await store.switchSubscription(target.value); }
  catch { target.value = store.activeSubscription; }
}
function openDiagnostics() {
  diagnosticsOpen.value = true;
  void store.runDiagnostics();
}
</script>

<template>
  <div class="dashboard">
    <section class="home-status" aria-labelledby="connection-title">
      <div class="home-status-main">
        <div class="home-status-copy">
          <div class="home-kicker"><span class="status-dot" :class="{ online: store.systemProxyEnabled || store.tunMode }"></span>连接状态</div>
          <h2 id="connection-title">{{ connectionSummary }}</h2>
          <p v-if="store.recoveryPending">原系统代理设置尚未恢复，请重试或检查网络设置。</p>
          <p v-else-if="store.systemProxyEnabled">遵循 Windows 系统代理的应用将使用本地代理端口。</p>
          <p v-else-if="store.tunMode">TUN 正在运行；关闭代理将同时停止内核与 TUN。</p>
          <p v-else-if="store.isConnected">Mihomo 已运行，Windows 系统代理尚未由本应用接管。</p>
          <p v-else>启动内核并开启 Windows 系统代理。</p>
        </div>
        <div class="home-primary-action"><ProxyToggle /><span>关闭时停止内核并恢复系统代理</span></div>
      </div>
      <div class="home-status-details">
        <div><Cpu :size="16" /><span>核心</span><strong :class="{ 'state-ok': store.isConnected }">{{ coreStatus }}</strong></div>
        <div><Shield :size="16" /><span>Windows 系统代理</span><strong :class="{ 'state-ok': store.systemProxyEnabled }">{{ proxyStatus }}</strong></div>
        <div><Globe2 :size="16" /><span>TUN</span><strong :class="{ 'state-ok': store.tunMode }">{{ store.tunMode ? '运行中' : '未开启' }}</strong></div>
        <div><Layers :size="16" /><span>代理模式</span><strong>{{ modes.find(mode => mode.value === store.proxyMode)?.label || '未知' }}</strong></div>
      </div>
      <div v-if="store.error" class="home-status-recovery">
        <TriangleAlert :size="15" /><span>运行异常</span>
        <button @click="openDiagnostics">检查网络 <ArrowRight :size="14" /></button>
        <button @click="store.setCurrentTab('logs')">查看日志 <ArrowRight :size="14" /></button>
      </div>
    </section>

    <div class="home-overview">
      <section class="panel home-config" aria-labelledby="config-title">
        <div class="section-heading"><h2 id="config-title"><Layers :size="17" />当前订阅与策略</h2><button class="text-button" @click="store.setCurrentTab('proxies')">管理订阅<ArrowRight :size="14" /></button></div>
        <div class="home-field">
          <div class="home-field-heading"><label for="active-subscription">当前订阅</label><span v-if="store.subscriptionBusy"><LoaderCircle :size="12" class="animate-spin" />切换中</span><span v-else-if="activeSubscription?.status === 'ready'" class="state-ok">已加载</span><span v-else>{{ activeSubscription?.status === 'error' ? '加载失败' : activeSubscription ? '待加载' : '未添加' }}</span></div>
          <select v-if="store.subscriptions.length" id="active-subscription" class="node-select" :value="store.activeSubscription" :title="store.activeSubscription" @change="selectSubscription" :disabled="store.subscriptionBusy || store.isBusy">
            <option v-for="sub in store.subscriptions" :key="sub.name" :value="sub.name">{{ sub.name }}</option>
          </select>
          <button v-else class="home-empty-link" @click="store.setCurrentTab('proxies')">添加订阅 <ArrowRight :size="14" /></button>
          <p v-if="activeSubscription && activeSubscription.updateTime !== '尚未更新'" class="home-field-note">上次更新 {{ activeSubscription.updateTime }}</p>
        </div>
        <div class="home-field home-strategy">
          <div class="home-field-heading"><label for="active-node">{{ currentGroup?.name || '代理组' }}</label><button class="text-button" @click="store.setCurrentTab('groups')">管理策略<ArrowRight :size="14" /></button></div>
          <p v-if="store.proxyMode === 'direct'" class="home-direct-state">直连模式 · 不使用代理节点</p>
          <template v-else-if="currentGroup">
            <select id="active-node" class="node-select" :value="selected" :title="selected" @change="selectNode" :disabled="switching || store.subscriptionBusy || currentGroup.type !== 'Selector' || !store.isConnected">
              <option v-for="name in currentGroup.options" :key="name" :value="name">{{ name }}</option>
            </select>
            <div class="home-node-meta">
              <span>{{ nodeIsDirect ? '直接连接' : nodeIsGroup ? '由下级策略组选择' : currentGroup.type === 'Selector' ? '手动选择' : `${currentGroup.type} 自动策略` }}</span>
              <button v-if="currentNode" class="text-button" @click="store.testNode(currentNode.name)" :disabled="!!store.nodeTesting || store.subscriptionBusy"><RefreshCw :size="13" :class="{ 'animate-spin': store.nodeTesting === currentNode.name }" />{{ store.nodeTesting === currentNode.name ? '测速中' : nodeDelay }}</button>
            </div>
          </template>
          <p v-else class="home-field-note">{{ store.isConnected ? '暂无可用策略组' : '内核启动后显示策略组与节点' }}</p>
        </div>
      </section>

      <section class="panel home-traffic" aria-labelledby="traffic-title">
        <div class="section-heading"><h2 id="traffic-title"><Activity :size="17" />实时流量</h2><span class="micro-label">{{ store.trafficAvailable ? '实时更新' : '实时数据不可用' }}</span></div>
        <div class="home-speed-grid">
          <div><span><ArrowDown :size="16" />下载</span><strong>{{ store.trafficAvailable ? store.trafficData.down : '--' }}</strong></div>
          <div><span><ArrowUp :size="16" />上传</span><strong>{{ store.trafficAvailable ? store.trafficData.up : '--' }}</strong></div>
        </div>
        <div class="home-traffic-total"><span>本次内核运行累计</span><strong>{{ store.connectionsAvailable ? `下载 ${store.formatBytes(store.trafficTotal.down)} · 上传 ${store.formatBytes(store.trafficTotal.up)}` : '统计不可用' }}</strong></div>
        <div class="home-traffic-total"><span><Clock3 :size="14" />运行时长</span><strong>{{ uptime }}</strong></div>
      </section>
    </div>

    <section class="home-mode" aria-labelledby="mode-title">
      <div><h2 id="mode-title"><Gauge :size="17" />代理模式</h2><p>{{ store.proxyMode === 'global' ? '流量交由全局策略处理' : store.proxyMode === 'direct' ? '流量不通过代理节点转发' : store.proxyMode === 'rule' ? '流量按配置规则分流' : '启动内核后读取当前模式' }}</p></div>
      <div class="mode-segments" role="group" aria-label="代理模式">
        <button v-for="mode in modes" :key="mode.value" :class="{ active: store.proxyMode === mode.value }" :aria-pressed="store.proxyMode === mode.value" :disabled="!store.isConnected || store.modeBusy || store.subscriptionBusy" @click="store.changeProxyMode(mode.value)">{{ mode.label }}</button>
      </div>
    </section>

    <div class="home-actions">
      <button class="home-action" @click="store.setCurrentTab('connections')"><Activity :size="18" /><span><strong>{{ store.connectionsAvailable ? `${store.connections.length} 个活跃连接` : '连接数据不可用' }}</strong><small>查看连接</small></span><ArrowRight :size="16" /></button>
      <button class="home-action" :aria-expanded="diagnosticsOpen" @click="openDiagnostics"><SearchCheck :size="18" /><span><strong>{{ store.diagnosticBusy ? '正在检查网络' : '检查网络' }}</strong><small>核心、端口、DNS 与目标连通性</small></span><LoaderCircle v-if="store.diagnosticBusy" :size="16" class="animate-spin" /><ArrowRight v-else :size="16" /></button>
    </div>

    <section v-if="diagnosticsOpen" class="home-diagnostics" aria-labelledby="diagnostics-title">
      <div class="section-heading"><h2 id="diagnostics-title"><SearchCheck :size="17" />网络诊断</h2><button class="text-button" :disabled="store.diagnosticBusy" @click="store.runDiagnostics()"><RefreshCw :size="14" :class="{ 'animate-spin': store.diagnosticBusy }" />重新检测</button></div>
      <p class="home-field-note">DNS 测试 example.com；经本地代理访问 www.gstatic.com/generate_204。结果仅代表这两个测试目标。</p>
      <div class="diagnostic-grid">
        <div v-for="check in diagnosticRows" :key="check.key" class="diagnostic-item" :class="`diagnostic-${check.state}`">
          <CheckCircle2 v-if="check.state === 'success'" :size="16" /><TriangleAlert v-else-if="check.state === 'failed'" :size="16" /><CircleHelp v-else-if="check.state === 'untested'" :size="16" /><WifiOff v-else :size="16" />
          <div><strong>{{ check.label }}</strong><span>{{ check.detail }}</span></div>
        </div>
      </div>
      <div v-if="diagnosticRows.some(check => check.state === 'failed')" class="diagnostic-links"><button class="text-button" @click="store.setCurrentTab('logs')">查看日志<ArrowRight :size="14" /></button><button class="text-button" @click="store.setCurrentTab('settings')"><Settings2 :size="14" />网络设置</button></div>
    </section>
  </div>
</template>
