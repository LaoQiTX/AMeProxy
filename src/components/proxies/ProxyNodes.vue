<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from 'vue';
import { RefreshCw, Plus, Trash2, ChevronRight, Loader2, Link, Server } from 'lucide-vue-next';
import { testProxy, getProviderProxies } from '../../services/proxy';
import { useProxyStore } from '../../stores/proxyStore';

const proxyStore = useProxyStore();
const importDialog = ref<HTMLDialogElement>();
const newSubName = ref('');
const newSubUrl = ref('');
const importLoading = ref(false);
const selectedProvider = ref<string | null>(null);
const providerProxies = ref<Array<{ name: string; type: string; latency?: number }>>([]);
const proxiesLoading = ref(false);
const error = ref<string | null>(null);
const subscriptionList = computed(() => proxyStore.subscriptions);
const statusText = (status?: string) => ({ ready: '已加载', saved: '已保存', loading: '等待加载', error: '加载未完成' }[status || 'saved']);
let requestId = 0;

const loadProviderProxies = async (name: string) => {
  const current = ++requestId;
  selectedProvider.value = name;
  error.value = null;
  proxiesLoading.value = true;
  providerProxies.value = [];
  if (name !== proxyStore.activeSubscription) {
    proxiesLoading.value = false;
    return;
  }
  try {
    if (!proxyStore.isConnected) throw new Error('请先启动内核');
    const provider = await getProviderProxies(name);
    if (current !== requestId) return;
    providerProxies.value = (provider.proxies || []).map(p => ({
      name: p.name, type: p.type, latency: p.history?.at(-1)?.delay,
    }));
  } catch (err) {
    if (current !== requestId) return;
    error.value = String(err);
    providerProxies.value = [];
  } finally {
    if (current === requestId) proxiesLoading.value = false;
  }
};

const handleTestProxy = async (name: string) => {
  if (proxyStore.subscriptionBusy || selectedProvider.value !== proxyStore.activeSubscription) return;
  const current = requestId;
  try {
    const latency = await testProxy(name);
    if (current !== requestId) return;
    const proxy = providerProxies.value.find(p => p.name === name);
    if (proxy) proxy.latency = latency;
  } catch (err) { proxyStore.error = String(err); }
};

const handleChangeProxy = async (name: string) => {
  if (proxyStore.subscriptionBusy || selectedProvider.value !== proxyStore.activeSubscription) return;
  const groups = proxyStore.proxyGroups.filter(g => g.type === 'Selector' && g.options.includes(name));
  const group = groups.find(g => g.name === '默认') || groups.find(g => g.name !== 'GLOBAL') || groups[0];
  if (!group) {
    proxyStore.error = '未找到包含该节点的手动策略组';
    return;
  }
  await proxyStore.switchProxy(group.name, name).catch(() => {});
};

const activateSubscription = async (name: string) => {
  try {
    await proxyStore.switchSubscription(name);
    await loadProviderProxies(name);
  } catch { /* Keep the previous active subscription visible on failure. */ }
};

watch(() => [proxyStore.activeSubscription, proxyStore.isConnected] as const, ([name]) => {
  requestId++;
  providerProxies.value = [];
  if (name) void loadProviderProxies(name);
  else selectedProvider.value = null;
});

const handleImportSubscription = async () => {
  if (!newSubName.value || !newSubUrl.value || importLoading.value) return;
  importLoading.value = true;
  try {
    await proxyStore.importSubscription(newSubName.value.trim(), newSubUrl.value.trim());
    importDialog.value?.close();
    newSubName.value = '';
    newSubUrl.value = '';
  } catch { /* Keep form values and show the store error. */ }
  finally { importLoading.value = false; }
};

const handleDeleteSubscription = async (name: string) => {
  const suffix = name === proxyStore.activeSubscription ? '当前正在使用此订阅，删除后将切换到剩余订阅；没有剩余订阅时回到直连。' : '';
  if (!confirm('确定要删除订阅 "' + name + '" 吗？' + suffix)) return;
  try {
    await proxyStore.removeSubscription(name);
    if (selectedProvider.value === name) {
      requestId++;
      selectedProvider.value = null;
      providerProxies.value = [];
    }
  } catch { /* Store shows the error. */ }
};

onMounted(async () => {
  if (proxyStore.previewMode) return;
  try {
    await proxyStore.fetchProviders();
    if (proxyStore.activeSubscription) await loadProviderProxies(proxyStore.activeSubscription);
  } catch (err) { proxyStore.error = String(err); }
});
onUnmounted(() => { requestId++; });
</script>

<template>
  <div class="subscription-layout">
    <!-- 左侧订阅列表 -->
    <div class="subscription-sidebar">
      <div class="flex items-center justify-between mb-4">
        <h3 class="text-sm font-semibold text-gray-500 uppercase tracking-widest">订阅管理</h3>
        <button @click="importDialog?.showModal()" aria-label="导入订阅" class="p-2 bg-white rounded-xl shadow-none border border-[var(--line)] text-[var(--accent)] hover:bg-[var(--accent-soft)] transition-colors">
          <Plus class="w-5 h-5" />
        </button>
      </div>
      <p class="text-xs text-gray-500 leading-relaxed mb-4">每次仅启用一个订阅。切换会断开旧连接，应用将重新连接。</p>
      
      <!-- 订阅列表 -->
      <div class="space-y-2">
        <div 
          v-for="sub in subscriptionList" 
          :key="sub.name"
          class="p-4 bg-white rounded-lg shadow-none border transition-all"
          :class="selectedProvider === sub.name ? 'border-[var(--accent)] bg-[var(--accent-soft)]' : 'border-[var(--line)] hover:border-[var(--accent-border)]'"
        >
          <div class="flex items-start justify-between">
            <div class="flex-1 min-w-0">
              <button class="flex items-center gap-2 w-full text-left" @click="loadProviderProxies(sub.name)" :aria-label="'查看订阅 ' + sub.name">
                <Server class="w-4 h-4 text-[var(--accent)] flex-shrink-0" />
                <h4 class="font-semibold text-gray-800 text-sm truncate">{{ sub.name }}</h4>
              </button>
              <div class="mt-3 flex items-center justify-between gap-2">
                <span v-if="sub.name === proxyStore.activeSubscription" class="text-xs font-semibold text-[var(--accent)]">当前使用</span>
                <span v-else class="text-xs text-gray-500">未启用</span>
                <button v-if="sub.name !== proxyStore.activeSubscription" class="text-button" @click="activateSubscription(sub.name)" :disabled="proxyStore.subscriptionBusy || proxyStore.isBusy" :aria-label="'使用订阅 ' + sub.name">{{ proxyStore.subscriptionBusy ? '处理中…' : '使用此订阅' }}</button>
              </div>
              <div class="flex items-center space-x-1 mt-2">
                <Link class="w-3 h-3 text-gray-500 flex-shrink-0" />
                <p class="text-xs text-gray-500 truncate">{{ sub.url || '无链接' }}</p>
              </div>
              <div class="flex items-center justify-between mt-2">
                <span class="text-xs text-gray-500">{{ sub.name === proxyStore.activeSubscription ? sub.count + ' 个节点 · ' + statusText(sub.status) : '已保存，切换后加载' }}</span>
                <button 
                  @click.stop="handleDeleteSubscription(sub.name)" :aria-label="'删除订阅 ' + sub.name" :disabled="proxyStore.subscriptionBusy || proxyStore.isBusy"
                  class="p-1 text-gray-500 hover:text-red-500 transition-colors"
                >
                  <Trash2 class="w-4 h-4" />
                </button>
              </div>
            </div>
          </div>
        </div>
        
        <!-- 无订阅提示 -->
        <div v-if="subscriptionList.length === 0" class="p-4 bg-white/50 rounded-lg border border-[var(--line)] border-dashed text-center">
          <p class="text-sm text-gray-500">暂无订阅</p>
          <p class="text-xs text-gray-300 mt-1">点击右上角添加</p>
        </div>
      </div>
    </div>

    <!-- 右侧节点列表 -->
    <div class="flex-1 min-w-0">
      <div class="flex items-center justify-between mb-4">
        <div class="flex items-center space-x-4">
          <h3 class="text-lg font-semibold text-gray-800">
            {{ selectedProvider || '请选择订阅' }}
          </h3>
          <span v-if="selectedProvider === proxyStore.activeSubscription" class="text-sm text-gray-500">{{ providerProxies.length }} 个节点</span>
        </div>
        <button 
          v-if="selectedProvider && selectedProvider === proxyStore.activeSubscription"
          @click="loadProviderProxies(selectedProvider)" 
          :disabled="proxiesLoading || proxyStore.subscriptionBusy"
          class="flex items-center space-x-2 text-sm font-semibold text-gray-500 hover:text-gray-600 disabled:opacity-50"
        >
          <RefreshCw :class="['w-4 h-4', proxiesLoading ? 'animate-spin' : '']" />
          <span>{{ proxiesLoading ? '加载中...' : '刷新' }}</span>
        </button>
      </div>

      <div v-if="selectedProvider && selectedProvider !== proxyStore.activeSubscription" class="panel text-center py-12">
        <Server class="w-10 h-10 mx-auto mb-3 text-gray-400" />
        <h4 class="font-semibold mb-2">此订阅尚未启用</h4>
        <p class="text-xs text-gray-500 mb-5">切换后仅加载此订阅的节点，原订阅保留在列表中。</p>
        <button class="proxy-toggle" @click="activateSubscription(selectedProvider)" :disabled="proxyStore.subscriptionBusy || proxyStore.isBusy">{{ proxyStore.subscriptionBusy ? '切换中…' : '使用此订阅' }}</button>
      </div>
      <!-- 未选择订阅 -->
      <div v-else-if="!selectedProvider" class="flex flex-col items-center justify-center py-16 bg-white/50 rounded-xl border border-[var(--line)] border-dashed text-gray-500">
        <Server class="w-12 h-12 mb-2" />
        <p class="text-sm font-semibold">请选择订阅</p>
        <p class="text-xs mt-1">选择一个订阅，查看可用节点与连接延迟</p>
      </div>

      <!-- 加载节点中 -->
      <div v-else-if="proxiesLoading" class="flex flex-col items-center justify-center py-16 bg-white/50 rounded-xl border border-[var(--line)] border-dashed text-gray-500">
        <Loader2 class="w-8 h-8 animate-spin mb-2" />
        <p class="text-sm font-semibold">加载节点中...</p>
      </div>

      <!-- 错误 -->
      <div v-else-if="error" class="flex flex-col items-center justify-center py-16 bg-white/50 rounded-xl border border-[var(--line)] border-dashed text-red-400">
        <p class="text-sm font-semibold">加载失败</p>
        <p class="text-xs mt-1">{{ error }}</p>
        <button 
          @click="loadProviderProxies(selectedProvider!)" 
          class="mt-4 px-4 py-2 bg-[var(--accent)] text-white rounded-lg text-sm"
        >
          重试
        </button>
      </div>

      <!-- 无节点 -->
      <div v-else-if="providerProxies.length === 0" class="flex flex-col items-center justify-center py-16 bg-white/50 rounded-xl border border-[var(--line)] border-dashed text-gray-500">
        <p class="text-sm font-semibold">暂无代理节点</p>
        <p class="text-xs mt-1">该订阅下没有节点</p>
      </div>

      <!-- 节点列表 -->
      <div v-else class="grid grid-cols-1 xl:grid-cols-2 gap-3">
        <div 
          v-for="proxy in providerProxies" 
          :key="proxy.name"
          @click="handleChangeProxy(proxy.name)" role="button" tabindex="0" :aria-label="'选择节点 ' + proxy.name" @keydown.enter.self="handleChangeProxy(proxy.name)" @keydown.space.prevent.self="handleChangeProxy(proxy.name)"
          class="bg-white p-4 rounded-lg shadow-none border border-[var(--line)] flex items-center justify-between hover:border-[var(--accent-border)] transition-shadow cursor-pointer group"
        >
          <div class="flex items-center space-x-3">
            <div class="w-10 h-10 bg-gray-50 rounded-xl flex items-center justify-center font-semibold text-gray-500 group-hover:bg-[var(--accent-soft)] group-hover:text-[var(--accent)] transition-colors text-sm">
              {{ proxy.name.charAt(0).toUpperCase() }}
            </div>
            <div class="min-w-0">
              <h4 class="font-semibold text-gray-800 text-sm truncate">{{ proxy.name }}</h4>
              <p class="text-xs text-gray-500">{{ proxy.type }}</p>
            </div>
          </div>
          <div class="flex items-center space-x-2">
            <button @click.stop="handleTestProxy(proxy.name)" :disabled="proxyStore.subscriptionBusy" title="测试延迟" :class="['text-xs font-semibold', proxy.latency ? (proxy.latency < 100 ? 'text-[var(--accent)]' : 'text-amber-500') : 'text-gray-300']">
              {{ proxy.latency ? proxy.latency + 'ms' : '测速' }}
            </button>
            <ChevronRight class="w-4 h-4 text-gray-300" />
          </div>
        </div>
      </div>
    </div>

    <dialog ref="importDialog" class="edit-dialog" aria-labelledby="import-dialog-title" @cancel="importLoading && $event.preventDefault()">
      <form @submit.prevent="handleImportSubscription">
        <div class="section-heading"><h2 id="import-dialog-title">导入订阅</h2></div>
        <p class="text-xs text-gray-500 mb-5 leading-relaxed">支持 Clash YAML 订阅。首个订阅自动选用，之后导入仅保存；点击“使用此订阅”切换。系统代理由主开关控制。</p>
        <label for="import-name" class="form-label">订阅名称</label>
        <input id="import-name" v-model="newSubName" class="form-input" placeholder="例如：日常使用" required :disabled="importLoading" />
        <label for="import-url" class="form-label">订阅链接</label>
        <input id="import-url" v-model="newSubUrl" class="form-input" type="url" placeholder="https://" required :disabled="importLoading" />
        <p v-if="proxyStore.error" role="alert" class="form-error">{{ proxyStore.error }}</p>
        <div class="dialog-actions">
          <button type="button" class="secondary-button" @click="importDialog?.close()" :disabled="importLoading">取消</button>
          <button type="submit" class="proxy-toggle" :disabled="importLoading || proxyStore.isBusy || proxyStore.previewMode"><Loader2 v-if="importLoading" :size="15" class="animate-spin" />{{ importLoading ? '导入中…' : '导入订阅' }}</button>
        </div>
      </form>
    </dialog>
  </div>
</template>
