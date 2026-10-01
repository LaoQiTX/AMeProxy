<script setup lang="ts">
import { computed, ref } from 'vue';
import { ArrowDown, ArrowUp, ArrowRight, Activity, Clock3, Globe2, Layers, RefreshCw, Network, Monitor, Server, Check } from 'lucide-vue-next';
import { useProxyStore } from '../../stores/proxyStore';
import ProxyToggle from '../common/ProxyToggle.vue';
const store = useProxyStore();
const switching = ref(false);
const defaultGroup = computed(() => store.proxyGroups.find(g => g.name === '默认') || store.proxyGroups.find(g => g.type === 'Selector' && g.name !== 'GLOBAL') || store.proxyGroups.find(g => g.name === 'GLOBAL') || store.proxyGroups[0]);
const selected = computed(() => defaultGroup.value?.selected || '');
const currentProxy = computed(() => store.proxies.find(p => p.name === selected.value));
const direct = computed(() => ['DIRECT', '直连'].includes(selected.value));
const uptime = computed(() => {
  if (!store.isConnected) return '—';
  const s = store.uptime;
  return [Math.floor(s / 3600), Math.floor(s % 3600 / 60), s % 60].map(n => String(n).padStart(2, '0')).join(':');
});
const delay = computed(() => !currentProxy.value?.delay ? '尚未测速' : currentProxy.value.delay < 0 ? '连接超时' : currentProxy.value.delay + ' ms');
async function selectNode(event: Event) {
  const group = defaultGroup.value;
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
</script>

<template>
  <div class="dashboard">
    <section class="connection-panel" :class="{ 'is-active': store.systemProxyEnabled }" aria-labelledby="connection-title">
      <div class="connection-main">
        <div class="connection-copy">
          <span class="eyebrow"><i class="status-dot" :class="{ online: store.systemProxyEnabled }"></i> SYSTEM PROXY</span>
          <h2 id="connection-title">{{ store.systemProxyEnabled ? '代理已开启' : store.tunMode ? 'TUN 已开启' : '准备好，连接世界' }}</h2>
          <p>{{ store.systemProxyEnabled ? '系统代理已指向本地内核，遵循系统设置的应用将按规则转发。' : store.tunMode ? '虚拟网卡正在运行，关闭代理将同时停止内核与 TUN。' : '一键启动内核并设置系统代理，无需手动配置浏览器。' }}</p>
          <div class="connection-actions"><ProxyToggle /><span>{{ store.systemProxyEnabled ? '关闭时恢复原系统设置' : 'Windows 系统代理' }}</span></div>
        </div>
        <div class="connection-visual" aria-hidden="true">
          <div class="orbit orbit-outer"></div><div class="orbit orbit-inner"></div>
          <div class="network-symbol"><Network :size="38" :stroke-width="1.5" /></div>
          <span class="orbit-node node-a"></span><span class="orbit-node node-b"></span>
        </div>
      </div>
      <div class="connection-path">
        <span><Monitor :size="14" />本机应用</span><span class="path-line"></span>
        <span :class="{ 'accent-text': store.systemProxyEnabled }"><Check v-if="store.systemProxyEnabled" :size="14" /><Server v-else :size="14" />系统代理</span><span class="path-line"></span>
        <span><Globe2 :size="14" />{{ direct ? '直连出口' : '按规则分流' }}</span>
      </div>
    </section>

    <div class="overview-grid">
      <section class="panel node-panel">
        <div class="section-heading"><h2><Layers :size="17" />当前策略</h2><button class="text-button" @click="store.setCurrentTab('groups')">管理策略<ArrowRight :size="14" /></button></div>
        <div v-if="store.subscriptions.length" class="mb-4">
          <label for="active-subscription" class="micro-label">当前订阅{{ store.subscriptionBusy ? ' · 处理中…' : '' }}</label>
          <select id="active-subscription" class="node-select" :value="store.activeSubscription" @change="selectSubscription" :disabled="store.subscriptionBusy || store.isBusy">
            <option v-for="sub in store.subscriptions" :key="sub.name" :value="sub.name">{{ sub.name }}</option>
          </select>
          <p v-if="store.subscriptions.length > 1" class="micro-label mt-2">仅使用选中订阅；切换会断开旧连接。</p>
        </div>
        <template v-if="defaultGroup">
          <div class="node-summary"><span class="node-avatar"><Globe2 :size="23" /></span><div><p class="micro-label">{{ defaultGroup.name }}</p><h3>{{ selected || '尚未选择节点' }}</h3></div></div>
          <label for="active-node" class="micro-label">切换出口节点</label>
          <select id="active-node" class="node-select" :value="selected" @change="selectNode" :disabled="switching || store.subscriptionBusy || defaultGroup.type !== 'Selector' || !store.isConnected">
            <option v-for="name in defaultGroup.options" :key="name" :value="name">{{ name }}</option>
          </select>
          <div class="node-meta"><span>{{ direct ? 'DIRECT · 直接连接' : (currentProxy?.type || defaultGroup.type) }}</span><button class="text-button" @click="store.testLatency()" :disabled="store.isTesting || !store.proxies.length"><RefreshCw :size="13" :class="{ 'animate-spin': store.isTesting }" />{{ store.isTesting ? '测速中' : delay }}</button></div>
          <p v-if="direct" class="inline-hint">当前出口为直连。选择订阅节点后，匹配代理规则的流量才会经过远程节点。</p>
        </template>
        <div v-else class="empty-node">
          <div class="empty-symbol"><Globe2 :size="26" :stroke-width="1.5" /></div>
          <h3>{{ store.subscriptions.length ? '启动后加载节点' : '从一个订阅开始' }}</h3>
          <p>导入 Clash YAML 订阅，再选择你的连接节点。</p>
          <button class="secondary-button" @click="store.setCurrentTab('proxies')">{{ store.subscriptions.length ? '查看订阅' : '添加订阅' }}<ArrowRight :size="14" /></button>
        </div>
      </section>

      <section class="panel traffic-panel">
        <div class="section-heading"><h2><Activity :size="17" />实时流量</h2><span class="micro-label">内核统计</span></div>
        <div class="traffic-row"><span class="traffic-icon"><ArrowDown :size="19" /></span><div><span class="micro-label">下载速度</span><strong>{{ store.trafficData.down }}</strong></div><span class="traffic-total">累计 {{ store.formatBytes(store.trafficTotal.down) }}</span></div>
        <div class="traffic-row"><span class="traffic-icon upload"><ArrowUp :size="19" /></span><div><span class="micro-label">上传速度</span><strong>{{ store.trafficData.up }}</strong></div><span class="traffic-total">累计 {{ store.formatBytes(store.trafficTotal.up) }}</span></div>
        <div class="session-meta"><span><Clock3 :size="14" />内核运行时长</span><strong>{{ uptime }}</strong></div>
      </section>
    </div>

    <button class="activity-strip" @click="store.setCurrentTab('connections')">
      <span class="activity-icon"><Activity :size="18" /></span>
      <span><strong>{{ store.connections.length }} 个活跃连接</strong><small>查看应用请求、命中规则与流量去向</small></span>
      <ArrowRight :size="18" />
    </button>
  </div>
</template>
