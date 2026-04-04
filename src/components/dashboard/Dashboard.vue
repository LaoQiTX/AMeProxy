<script setup lang="ts">
import { Shield, Activity, Cpu, RefreshCw, Wifi, Palette, ArrowUp, ArrowDown, ChevronDown, ChevronUp } from 'lucide-vue-next';
import { useProxyStore } from '../../stores/proxyStore';
import { useThemeStore } from '../../stores/themeStore';
import { computed, ref } from 'vue';

const proxyStore = useProxyStore();
const themeStore = useThemeStore();

const showNodeSelector = ref(false);

const defaultGroup = computed(() => proxyStore.proxyGroups.find(g => g.name === '默认'));
const currentProxy = computed(() => {
  const selected = defaultGroup.value?.selected;
  return proxyStore.proxies.find(p => p.name === selected) || { name: selected || '未连接', type: '-', delay: 0 };
});

const currentDelay = computed(() => {
  if (currentProxy.value.delay > 0) return currentProxy.value.delay + ' ms';
  if (currentProxy.value.delay === -1) return 'Timeout';
  return '- ms';
});

const selectNode = async (nodeName: string) => {
  try {
    await proxyStore.switchProxy('默认', nodeName);
    showNodeSelector.value = false;
  } catch (error) {
    console.error('Failed to select node:', error);
  }
};

</script>

<template>
  <div class="space-y-8">
    <!-- Stats Cards -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
      <div class="bg-white rounded-3xl p-6 shadow-sm border border-gray-50 flex items-center space-x-4">
        <div class="p-4 bg-emerald-50 rounded-2xl">
          <Shield class="w-8 h-8 text-emerald-500" />
        </div>
        <div>
          <p class="text-sm text-gray-400 font-medium">代理状态</p>
          <p class="text-xl font-bold text-gray-800">{{ proxyStore.isConnected ? '已开启' : '未开启' }}</p>
        </div>
      </div>
      <div class="bg-white rounded-3xl p-6 shadow-sm border border-gray-50 flex items-center space-x-4">
        <div class="p-4 bg-blue-50 rounded-2xl">
          <ArrowDown class="w-8 h-8 text-blue-500" />
        </div>
        <div>
          <p class="text-sm text-gray-400 font-medium">下载速度</p>
          <p class="text-xl font-bold text-gray-800">{{ proxyStore.trafficData.down }}</p>
        </div>
      </div>
      <div class="bg-white rounded-3xl p-6 shadow-sm border border-gray-50 flex items-center space-x-4">
        <div class="p-4 bg-amber-50 rounded-2xl">
          <ArrowUp class="w-8 h-8 text-amber-500" />
        </div>
        <div>
          <p class="text-sm text-gray-400 font-medium">上传速度</p>
          <p class="text-xl font-bold text-gray-800">{{ proxyStore.trafficData.up }}</p>
        </div>
      </div>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
      <div class="lg:col-span-2 bg-white rounded-[2rem] p-8 shadow-sm border border-gray-50 relative overflow-hidden">
        <div class="relative z-10">
          <div class="flex items-center justify-between mb-8">
            <h3 class="text-lg font-bold text-gray-800">当前连接节点 (默认组)</h3>
            <button @click="proxyStore.testLatency()" class="p-2 hover:bg-gray-50 rounded-xl transition-colors">
              <RefreshCw :class="['w-5 h-5 text-gray-400', proxyStore.isTesting ? 'animate-spin' : '']" />
            </button>
          </div>
          
          <div class="flex items-center space-x-6">
            <div class="w-20 h-20 bg-gray-50 rounded-3xl flex items-center justify-center text-3xl font-bold text-gray-300">
              {{ (currentProxy.name || '未连接').substring(0, 2) }}
            </div>
            <div class="relative">
              <div @click="showNodeSelector = !showNodeSelector" class="flex items-center justify-between cursor-pointer">
                <div>
                  <h4 class="text-2xl font-black text-gray-800">{{ currentProxy.name }}</h4>
                  <div class="flex items-center space-x-3 mt-2">
                    <span class="px-2 py-0.5 bg-emerald-50 text-emerald-600 text-[10px] font-bold rounded uppercase">{{ currentProxy.type }}</span>
                    <span class="flex items-center text-emerald-500 text-sm font-bold">
                      <Wifi class="w-4 h-4 mr-1" /> {{ currentDelay }}
                    </span>
                  </div>
                </div>
                <div class="ml-4">
                  <ChevronDown v-if="!showNodeSelector" class="w-5 h-5 text-gray-400" />
                  <ChevronUp v-else class="w-5 h-5 text-gray-400" />
                </div>
              </div>
              <div v-if="showNodeSelector" class="absolute top-full left-0 mt-2 w-80 bg-white rounded-xl shadow-lg border border-gray-100 z-10 max-h-80 overflow-y-auto">
                <div class="p-2">
                  <div 
                    v-for="proxy in proxyStore.proxies" 
                    :key="proxy.name"
                    @click="selectNode(proxy.name)"
                    class="px-4 py-2 rounded-lg hover:bg-gray-50 cursor-pointer flex items-center justify-between"
                  >
                    <div class="flex items-center space-x-2">
                      <span class="w-6 h-6 bg-gray-100 rounded flex items-center justify-center text-xs font-bold">
                        {{ proxy.region || '?' }}
                      </span>
                      <span class="text-sm font-medium">{{ proxy.name }}</span>
                    </div>
                    <span :class="['text-xs font-bold', proxy.delay === -1 ? 'text-gray-400' : proxy.delay < 100 ? 'text-emerald-500' : 'text-amber-500']">
                      {{ proxy.delay === -1 ? '...' : proxy.delay + 'ms' }}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <div class="mt-12 grid grid-cols-2 gap-4">
            <div class="bg-gray-50/50 rounded-2xl p-4">
              <p class="text-xs text-gray-400 font-bold uppercase mb-1">今日流量</p>
              <p class="text-lg font-bold text-gray-700">-</p>
            </div>
            <div class="bg-gray-50/50 rounded-2xl p-4">
              <p class="text-xs text-gray-400 font-bold uppercase mb-1">本月流量</p>
              <p class="text-lg font-bold text-gray-700">-</p>
            </div>
          </div>
        </div>
        <div :class="['absolute -bottom-12 -right-12 w-64 h-64 rounded-full opacity-10 blur-3xl', themeStore.getCurrentTheme()?.text.replace('text', 'bg')]"></div>
      </div>

      <div class="bg-white rounded-[2rem] p-8 shadow-sm border border-gray-50">
        <h3 class="text-lg font-bold text-gray-800 mb-6 flex items-center">
          <Palette class="w-5 h-5 mr-2 text-gray-400" />
          个性化配色
        </h3>
        <div class="grid grid-cols-2 gap-4">
          <button 
            v-for="theme in themeStore.themes" 
            :key="theme.id"
            @click="themeStore.setTheme(theme.id)"
            :class="[
              'p-4 rounded-2xl border-2 transition-all duration-300 text-left group',
              themeStore.currentTheme === theme.id ? 'border-gray-800 bg-gray-50' : 'border-transparent bg-gray-50/50 hover:bg-gray-50'
            ]"
          >
            <div :style="{ backgroundColor: theme.color }" class="w-8 h-8 rounded-lg mb-3 shadow-sm group-hover:scale-110 transition-transform"></div>
            <p class="text-sm font-bold text-gray-700">{{ theme.name }}</p>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
