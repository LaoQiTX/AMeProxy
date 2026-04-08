<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue';
import { RefreshCw, Plus, Trash2, ChevronRight, Loader2, Link, Server } from 'lucide-vue-next';
import { testProxy, changeProxy, startCore, is_proxy_running, getProviders, getProviderProxies, triggerProviderHealthCheck } from '../../services/proxy';
import { useProxyStore } from '../../stores/proxyStore';

const proxyStore = useProxyStore();

const showImportSub = ref(false);
const newSubUrl = ref('');
const newSubName = ref('');
const importLoading = ref(false);
const providers = ref<Record<string, any>>({});
const selectedProvider = ref<string | null>(null);
const providerProxies = ref<any[]>([]);
const loading = ref(false);
const proxiesLoading = ref(false);
const error = ref<string | null>(null);

// 从 store 获取订阅列表（本地配置的订阅）
const subscriptionList = computed(() => {
  return proxyStore.subscriptions;
});

// 加载提供者节点
const loadProviderProxies = async (providerName: string) => {
  proxiesLoading.value = true;
  try {
    // 检查代理内核是否正在运行
    const isRunning = await is_proxy_running();
    if (!isRunning) {
      console.log('Starting proxy core...');
      await startCore();
      console.log('Proxy core started');
      // 等待内核初始化
      console.log('Waiting for kernel to initialize...');
      await new Promise(resolve => setTimeout(resolve, 5000));
      console.log('Kernel initialized');
      
      // 等待订阅下载和解析
      console.log('Waiting for subscription download and parse...');
      await new Promise(resolve => setTimeout(resolve, 5000));
      console.log('Subscription downloaded and parsed');
    }
    
    // 从 store 获取订阅信息
    console.log('Fetching providers from store...');
    await proxyStore.fetchProviders();
    console.log('Store subscriptions:', proxyStore.subscriptions);
    
    // 直接从 API 获取最新的提供者信息
    try {
      const response = await getProviders();
      console.log('Providers response:', response);
      
      // 处理 providers 数据
      if (response && (response as any).providers) {
        providers.value = (response as any).providers;
      } else if (response && typeof response === 'object') {
        providers.value = response as Record<string, any>;
      } else {
        providers.value = {};
      }
      console.log('Using real providers data:', providers.value);
    } catch (err) {
      console.error('Failed to get providers from API:', err);
      // API 调用失败，使用空对象
      providers.value = {};
      console.log('API call failed, using empty providers data');
    }
    
    // 加载选中提供者的节点
    selectProvider(providerName);
  } catch (err: any) {
    console.error('Failed to load providers:', err);
    error.value = err.message || '加载节点失败';
    providers.value = {};
    providerProxies.value = [];
  } finally {
    proxiesLoading.value = false;
  }
};

// 选择订阅并加载节点
const selectProvider = (name: string) => {
  selectedProvider.value = name;
  
  try {
    const provider = providers.value[name];
    console.log('Selected provider:', provider);
    
    if (provider && provider.proxies) {
      console.log('Provider proxies:', provider.proxies);
      providerProxies.value = provider.proxies.map((p: any) => ({
        name: p.name,
        type: p.type,
        latency: p.history?.[0]?.delay || p.extra?.['https://www.gstatic.com/generate_204']?.history?.[0]?.delay || undefined
      }));
      console.log('Mapped provider proxies:', providerProxies.value);
    } else {
      console.log('No proxies found for provider:', name);
      providerProxies.value = [];
    }
  } catch (err) {
    console.error('Failed to load provider proxies:', err);
    providerProxies.value = [];
  }
};

// 测试节点延迟
const handleTestProxy = async (proxyName: string) => {
  try {
    const latency = await testProxy(proxyName);
    const proxy = providerProxies.value.find(p => p.name === proxyName);
    if (proxy) {
      proxy.latency = latency;
    }
  } catch (error) {
    console.error('Failed to test proxy:', error);
  }
};

// 切换代理
const handleChangeProxy = async (proxyName: string) => {
  if (!selectedProvider.value) return;
  try {
    await changeProxy(selectedProvider.value, proxyName);
  } catch (error) {
    console.error('Failed to change proxy:', error);
  }
};

// 导入订阅
const handleImportSubscription = async () => {
  if (newSubName.value && newSubUrl.value) {
    importLoading.value = true;
    try {
      await proxyStore.importSubscription(newSubName.value, newSubUrl.value);
      showImportSub.value = false;
      newSubName.value = '';
      newSubUrl.value = '';
      
      // 导入完成后重新加载订阅列表
      console.log('Reloading subscriptions...');
      await proxyStore.fetchProviders();
      console.log('Subscriptions reloaded:', proxyStore.subscriptions);
    } catch (error) {
      console.error('Failed to import subscription:', error);
    } finally {
      importLoading.value = false;
    }
  }
};

// 删除订阅
const handleDeleteSubscription = async (name: string) => {
  if (confirm(`确定要删除订阅 "${name}" 吗？`)) {
    try {
      await proxyStore.removeSubscription(name);
      if (selectedProvider.value === name) {
        selectedProvider.value = null;
        providerProxies.value = [];
      }
    } catch (error) {
      console.error('Failed to delete subscription:', error);
    }
  }
};

onMounted(async () => {
  // 组件挂载时加载订阅列表
  console.log('Initializing subscriptions...');
  await proxyStore.fetchProviders();
  console.log('Initialized with subscriptions:', subscriptionList.value);
});
</script>

<template>
  <div class="flex h-full gap-6">
    <!-- 左侧订阅列表 -->
    <div class="w-80 flex-shrink-0">
      <div class="flex items-center justify-between mb-4">
        <h3 class="text-sm font-bold text-gray-400 uppercase tracking-widest">订阅管理</h3>
        <button @click="showImportSub = true" class="p-2 bg-white rounded-xl shadow-sm border border-gray-50 text-emerald-500 hover:bg-emerald-50 transition-colors">
          <Plus class="w-5 h-5" />
        </button>
      </div>
      
      <!-- 订阅列表 -->
      <div class="space-y-2">
        <div 
          v-for="sub in subscriptionList" 
          :key="sub.name"
          @click="loadProviderProxies(sub.name)"
          class="p-4 bg-white rounded-2xl shadow-sm border cursor-pointer transition-all"
          :class="selectedProvider === sub.name ? 'border-emerald-400 bg-emerald-50' : 'border-gray-50 hover:border-emerald-200'"
        >
          <div class="flex items-start justify-between">
            <div class="flex-1 min-w-0">
              <div class="flex items-center space-x-2">
                <Server class="w-4 h-4 text-emerald-500 flex-shrink-0" />
                <h4 class="font-bold text-gray-800 text-sm truncate">{{ sub.name }}</h4>
              </div>
              <div class="flex items-center space-x-1 mt-2">
                <Link class="w-3 h-3 text-gray-400 flex-shrink-0" />
                <p class="text-xs text-gray-400 truncate">{{ sub.url || '无链接' }}</p>
              </div>
              <div class="flex items-center justify-between mt-2">
                <span class="text-xs text-gray-400">{{ sub.count }} 个节点</span>
                <button 
                  @click.stop="handleDeleteSubscription(sub.name)"
                  class="p-1 text-gray-400 hover:text-red-500 transition-colors"
                >
                  <Trash2 class="w-4 h-4" />
                </button>
              </div>
            </div>
          </div>
        </div>
        
        <!-- 无订阅提示 -->
        <div v-if="subscriptionList.length === 0" class="p-4 bg-white/50 rounded-2xl border border-gray-50 border-dashed text-center">
          <p class="text-sm text-gray-400">暂无订阅</p>
          <p class="text-xs text-gray-300 mt-1">点击右上角添加</p>
        </div>
      </div>
    </div>

    <!-- 右侧节点列表 -->
    <div class="flex-1">
      <div class="flex items-center justify-between mb-4">
        <div class="flex items-center space-x-4">
          <h3 class="text-lg font-bold text-gray-800">
            {{ selectedProvider || '请选择订阅' }}
          </h3>
          <span v-if="selectedProvider" class="text-sm text-gray-400">{{ providerProxies.length }} 个节点</span>
        </div>
        <button 
          v-if="selectedProvider"
          @click="loadProviderProxies(selectedProvider)" 
          :disabled="proxiesLoading"
          class="flex items-center space-x-2 text-sm font-bold text-gray-400 hover:text-gray-600 disabled:opacity-50"
        >
          <RefreshCw :class="['w-4 h-4', proxiesLoading ? 'animate-spin' : '']" />
          <span>{{ proxiesLoading ? '加载中...' : '刷新' }}</span>
        </button>
      </div>

      <!-- 未选择订阅 -->
      <div v-if="!selectedProvider" class="flex flex-col items-center justify-center py-16 bg-white/50 rounded-3xl border border-gray-50 border-dashed text-gray-400">
        <Server class="w-12 h-12 mb-2" />
        <p class="text-sm font-bold">请选择订阅</p>
        <p class="text-xs mt-1">从左侧列表中选择一个订阅查看节点</p>
      </div>

      <!-- 加载节点中 -->
      <div v-else-if="proxiesLoading" class="flex flex-col items-center justify-center py-16 bg-white/50 rounded-3xl border border-gray-50 border-dashed text-gray-400">
        <Loader2 class="w-8 h-8 animate-spin mb-2" />
        <p class="text-sm font-bold">加载节点中...</p>
      </div>

      <!-- 错误 -->
      <div v-else-if="error" class="flex flex-col items-center justify-center py-16 bg-white/50 rounded-3xl border border-gray-50 border-dashed text-red-400">
        <p class="text-sm font-bold">加载失败</p>
        <p class="text-xs mt-1">{{ error }}</p>
        <button 
          @click="loadProviderProxies(selectedProvider!)" 
          class="mt-4 px-4 py-2 bg-emerald-500 text-white rounded-lg text-sm"
        >
          重试
        </button>
      </div>

      <!-- 无节点 -->
      <div v-else-if="providerProxies.length === 0" class="flex flex-col items-center justify-center py-16 bg-white/50 rounded-3xl border border-gray-50 border-dashed text-gray-400">
        <p class="text-sm font-bold">暂无代理节点</p>
        <p class="text-xs mt-1">该订阅下没有节点</p>
      </div>

      <!-- 节点列表 -->
      <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        <div 
          v-for="proxy in providerProxies" 
          :key="proxy.name"
          @click="handleChangeProxy(proxy.name)"
          @dblclick="handleTestProxy(proxy.name)"
          class="bg-white p-4 rounded-2xl shadow-sm border border-gray-50 flex items-center justify-between hover:shadow-md transition-shadow cursor-pointer group"
        >
          <div class="flex items-center space-x-3">
            <div class="w-10 h-10 bg-gray-50 rounded-xl flex items-center justify-center font-bold text-gray-400 group-hover:bg-emerald-50 group-hover:text-emerald-500 transition-colors text-sm">
              {{ proxy.name.charAt(0).toUpperCase() }}
            </div>
            <div class="min-w-0">
              <h4 class="font-bold text-gray-800 text-sm truncate">{{ proxy.name }}</h4>
              <p class="text-xs text-gray-400">{{ proxy.type }}</p>
            </div>
          </div>
          <div class="flex items-center space-x-2">
            <span :class="['text-xs font-bold', proxy.latency ? (proxy.latency < 100 ? 'text-emerald-500' : 'text-amber-500') : 'text-gray-300']">
              {{ proxy.latency ? proxy.latency + 'ms' : '-' }}
            </span>
            <ChevronRight class="w-4 h-4 text-gray-300" />
          </div>
        </div>
      </div>
    </div>

    <!-- Import Subscription Modal -->
    <transition name="fade">
      <div v-if="showImportSub" class="fixed inset-0 bg-black/20 backdrop-blur-sm z-50 flex items-center justify-center p-4" @click.self="showImportSub = false">
        <transition name="scale">
          <div v-if="showImportSub" class="bg-white rounded-[2.5rem] shadow-2xl w-full max-w-md overflow-hidden flex flex-col p-8">
            <h3 class="text-2xl font-black text-gray-800 mb-6">导入订阅</h3>
            <div class="space-y-4">
              <div>
                <label class="text-xs font-bold text-gray-400 uppercase mb-2 block">订阅名称</label>
                <input v-model="newSubName" type="text" placeholder="例如：我的机场" class="w-full bg-gray-50 border-none rounded-2xl px-5 py-3 text-sm focus:ring-2 focus:ring-emerald-400 outline-none" :disabled="importLoading" />
              </div>
              <div>
                <label class="text-xs font-bold text-gray-400 uppercase mb-2 block">订阅链接</label>
                <input v-model="newSubUrl" type="text" placeholder="https://..." class="w-full bg-gray-50 border-none rounded-2xl px-5 py-3 text-sm focus:ring-2 focus:ring-emerald-400 outline-none" :disabled="importLoading" />
              </div>
            </div>
            <div class="mt-8 flex space-x-4">
              <button @click="showImportSub = false" class="flex-1 py-3 rounded-2xl font-bold text-gray-400 hover:bg-gray-50 transition-colors" :disabled="importLoading">取消</button>
              <button @click="handleImportSubscription" class="flex-1 py-3 rounded-2xl font-bold text-white shadow-lg transition-all bg-emerald-500 flex items-center justify-center space-x-2" :disabled="importLoading || !newSubName || !newSubUrl">
                <Loader2 v-if="importLoading" class="w-4 h-4 animate-spin" />
                <span>{{ importLoading ? '导入中...' : '确认导入' }}</span>
              </button>
            </div>
          </div>
        </transition>
      </div>
    </transition>
  </div>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.scale-enter-active,
.scale-leave-active {
  transition: all 0.3s ease;
}

.scale-enter-from,
.scale-leave-to {
  opacity: 0;
  transform: scale(0.95);
}
</style>
