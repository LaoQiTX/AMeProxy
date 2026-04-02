<script setup lang="ts">
import { computed } from 'vue';
import { Zap, BarChart3 } from 'lucide-vue-next';
import { useProxyStore } from '../../stores/proxyStore';
import { useThemeStore } from '../../stores/themeStore';

const proxyStore = useProxyStore();
const themeStore = useThemeStore();

const toggleConnection = () => {
  proxyStore.toggleConnection();
};

const getTabTitle = computed(() => {
  switch (proxyStore.currentTab) {
    case 'dashboard': return '欢迎回来';
    case 'groups': return '策略组';
    case 'proxies': return '代理节点';
    case 'connections': return '实时连接';
    case 'rules': return '分流规则';
    case 'logs': return '运行日志';
    case 'settings': return '系统设置';
    default: return '';
  }
});
</script>

<template>
  <header class="h-20 bg-white/40 backdrop-blur-md flex items-center justify-between px-8 border-b border-white/20 shrink-0 z-10">
    <div class="flex items-center space-x-4">
      <h2 class="text-xl font-bold text-gray-800">
        {{ getTabTitle }}
      </h2>
      <div v-if="proxyStore.currentTab === 'dashboard'" class="px-3 py-1 bg-white/60 rounded-full text-xs font-medium text-gray-500 border border-gray-100">
        运行时间: 12:45:08
      </div>
    </div>

    <div class="flex items-center space-x-6">
      <!-- Traffic Stats -->
      <div class="hidden lg:flex items-center space-x-8">
        <div class="flex items-center space-x-2">
          <div class="p-2 bg-blue-50 rounded-lg">
            <BarChart3 class="w-4 h-4 text-blue-500" />
          </div>
          <div>
            <p class="text-[10px] text-gray-400 font-bold uppercase">下载</p>
            <p class="text-sm font-bold text-gray-700">{{ proxyStore.trafficData.down }}</p>
          </div>
        </div>
        <div class="flex items-center space-x-2">
          <div class="p-2 bg-purple-50 rounded-lg">
            <BarChart3 class="w-4 h-4 text-purple-500 rotate-180" />
          </div>
          <div>
            <p class="text-[10px] text-gray-400 font-bold uppercase">上传</p>
            <p class="text-sm font-bold text-gray-700">{{ proxyStore.trafficData.up }}</p>
          </div>
        </div>
      </div>

      <!-- Connection Toggle -->
      <button 
        @click="toggleConnection"
        :class="[
          'px-6 py-2.5 rounded-full font-bold text-sm shadow-lg transition-all duration-300 active:scale-95 flex items-center space-x-2',
          proxyStore.isConnected 
            ? 'bg-red-500 text-white hover:bg-red-600 shadow-red-200' 
            : (themeStore.getCurrentTheme()?.btn || 'bg-emerald-500') + ' text-white hover:opacity-90 ' + (themeStore.getCurrentTheme()?.shadow || 'shadow-emerald-200')
        ]"
      >
        <Zap class="w-4 h-4" />
        <span>{{ proxyStore.isConnected ? '断开连接' : '开启代理' }}</span>
      </button>
    </div>
  </header>
</template>
