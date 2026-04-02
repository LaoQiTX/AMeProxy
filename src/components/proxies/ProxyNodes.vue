<script setup lang="ts">
import { ref } from 'vue';
import { RefreshCw, Plus, Trash2, ChevronRight } from 'lucide-vue-next';
import { useProxyStore } from '../../stores/proxyStore';

const showImportSub = ref(false);
const newSubUrl = ref('');
const newSubName = ref('');
const proxyStore = useProxyStore();

const importSubscription = async () => {
  console.log('开始导入订阅:', newSubName.value, newSubUrl.value);
  console.log('proxyStore:', proxyStore);
  try {
    await proxyStore.importSubscription(newSubName.value, newSubUrl.value);
    console.log('订阅导入成功');
    newSubUrl.value = '';
    newSubName.value = '';
    showImportSub.value = false;
  } catch (error) {
    console.error('订阅导入失败:', error);
  }
};
</script>

<template>
  <div class="space-y-8">
    <!-- Subscriptions -->
    <section>
      <div class="flex items-center justify-between mb-4">
        <h3 class="text-sm font-bold text-gray-400 uppercase tracking-widest">订阅管理</h3>
        <button @click="showImportSub = true" class="p-2 bg-white rounded-xl shadow-sm border border-gray-50 text-emerald-500 hover:bg-emerald-50 transition-colors">
          <Plus class="w-5 h-5" />
        </button>
      </div>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div v-for="sub in proxyStore.subscriptions" :key="sub.name" class="bg-white p-5 rounded-3xl shadow-sm border border-gray-50 flex items-center justify-between group">
          <div class="flex items-center space-x-4">
            <div class="w-12 h-12 bg-emerald-50 rounded-2xl flex items-center justify-center">
              <RefreshCw class="w-6 h-6 text-emerald-500" />
            </div>
            <div>
              <h4 class="font-bold text-gray-800">{{ sub.name }}</h4>
              <p class="text-xs text-gray-400">{{ sub.count }} 个节点 · {{ sub.updateTime }}</p>
            </div>
          </div>
          <div class="flex space-x-2">
            <button @click="proxyStore.importSubscription(sub.name, sub.url)" title="更新/切换并应用" class="p-2 hover:bg-emerald-50 rounded-xl text-emerald-500 transition-colors"><RefreshCw class="w-4 h-4" /></button>
            <button @click="proxyStore.removeSubscription(sub.name)" class="p-2 hover:bg-red-50 rounded-xl text-red-400 transition-colors"><Trash2 class="w-4 h-4" /></button>
          </div>
        </div>
      </div>
    </section>

    <!-- Node List -->
    <section>
      <div class="flex items-center justify-between mb-4">
        <div class="flex space-x-2">
          <button class="px-4 py-2 bg-white rounded-xl text-sm font-bold text-gray-700 shadow-sm border border-gray-50">全部节点</button>
          <button class="px-4 py-2 text-sm font-bold text-gray-400 hover:text-gray-600">常用节点</button>
        </div>
        <button 
          @click="proxyStore.testLatency()" 
          :disabled="proxyStore.isTesting || proxyStore.proxies.length === 0"
          class="flex items-center space-x-2 text-sm font-bold text-gray-400 hover:text-gray-600 disabled:opacity-50"
        >
          <RefreshCw :class="['w-4 h-4', proxyStore.isTesting ? 'animate-spin' : '']" />
          <span>{{ proxyStore.isTesting ? '测试中...' : '测速' }}</span>
        </button>
      </div>

      <div v-if="proxyStore.proxies.length === 0" class="flex flex-col items-center justify-center py-16 bg-white/50 rounded-3xl border border-gray-50 border-dashed text-gray-400">
        <p class="text-sm font-bold">暂无代理节点</p>
        <p class="text-xs mt-1">请检查订阅是否导入成功并已开启代理</p>
      </div>

      <div v-else class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div 
          v-for="proxy in proxyStore.proxies" 
          :key="proxy.name"
          class="bg-white p-5 rounded-3xl shadow-sm border border-gray-50 flex items-center justify-between hover:shadow-md transition-shadow cursor-pointer group"
        >
          <div class="flex items-center space-x-4">
            <div class="w-12 h-12 bg-gray-50 rounded-2xl flex items-center justify-center font-bold text-gray-400 group-hover:bg-emerald-50 group-hover:text-emerald-500 transition-colors">
              {{ proxy.region }}
            </div>
            <div>
              <h4 class="font-bold text-gray-800">{{ proxy.name }}</h4>
              <p class="text-xs text-gray-400 font-medium">{{ proxy.type }}</p>
            </div>
          </div>
          <div class="flex items-center space-x-4">
            <span :class="['text-sm font-bold', proxy.delay === -1 ? 'text-gray-300' : proxy.delay < 100 ? 'text-emerald-500' : 'text-amber-500']">
              {{ proxy.delay === -1 ? '...' : proxy.delay + 'ms' }}
            </span>
            <ChevronRight class="w-5 h-5 text-gray-300" />
          </div>
        </div>
      </div>
    </section>

    <!-- Import Subscription Modal -->
    <transition name="fade">
      <div v-if="showImportSub" class="fixed inset-0 bg-black/20 backdrop-blur-sm z-50 flex items-center justify-center p-4" @click.self="showImportSub = false">
        <transition name="scale">
          <div v-if="showImportSub" class="bg-white rounded-[2.5rem] shadow-2xl w-full max-w-md overflow-hidden flex flex-col p-8">
            <h3 class="text-2xl font-black text-gray-800 mb-6">导入订阅</h3>
            <div class="space-y-4">
              <div>
                <label class="text-xs font-bold text-gray-400 uppercase mb-2 block">订阅名称</label>
                <input v-model="newSubName" type="text" placeholder="例如：我的机场" class="w-full bg-gray-50 border-none rounded-2xl px-5 py-3 text-sm focus:ring-2 focus:ring-emerald-400 outline-none" />
              </div>
              <div>
                <label class="text-xs font-bold text-gray-400 uppercase mb-2 block">订阅链接</label>
                <input v-model="newSubUrl" type="text" placeholder="https://..." class="w-full bg-gray-50 border-none rounded-2xl px-5 py-3 text-sm focus:ring-2 focus:ring-emerald-400 outline-none" />
              </div>
            </div>
            <div class="mt-8 flex space-x-4">
              <button @click="showImportSub = false" class="flex-1 py-3 rounded-2xl font-bold text-gray-400 hover:bg-gray-50 transition-colors">取消</button>
              <button @click="importSubscription" class="flex-1 py-3 rounded-2xl font-bold text-white shadow-lg transition-all bg-emerald-500">确认导入</button>
            </div>
          </div>
        </transition>
      </div>
    </transition>
  </div>
</template>
