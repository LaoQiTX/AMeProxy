<script setup lang="ts">
import { ref } from 'vue';
import { 
  LayoutDashboard, 
  Globe, 
  Settings, 
  Zap, 
  Activity, 
  Shield, 
  Cpu,
  RefreshCw,
  MoreVertical,
  ChevronRight,
  Wifi,
  BarChart3,
  Moon,
  Sun,
  Palette,
  Layers,
  ListFilter,
  Terminal,
  Search,
  Trash2,
  Plus,
  ExternalLink,
  Info,
  AlertCircle
} from 'lucide-vue-next';
import { useProxyStore } from '../../stores/proxyStore';
import { useThemeStore } from '../../stores/themeStore';

const showHelp = ref(false);
const proxyStore = useProxyStore();
const themeStore = useThemeStore();

const toggleHelp = () => {
  showHelp.value = !showHelp.value;
};

const setCurrentTab = (tab: string) => {
  proxyStore.setCurrentTab(tab);
};
</script>

<template>
  <aside class="w-20 md:w-64 h-full bg-white/80 backdrop-blur-xl border-r border-gray-100 flex flex-col items-center py-8 px-4 shadow-sm z-20 shrink-0">
    <!-- Logo -->
    <div class="mb-10 flex flex-col items-center group cursor-pointer shrink-0">
      <div :class="['w-12 h-12 rounded-2xl flex items-center justify-center shadow-lg transition-transform group-hover:scale-110 group-hover:rotate-6', themeStore.getCurrentTheme()?.text.replace('text', 'bg').replace('600', '400')]">
        <img src="/icon.png" alt="AMeProxy">
      </div>
      <span class="mt-3 font-bold text-gray-800 hidden md:block text-lg">AMeProxy</span>
    </div>

    <!-- Nav -->
    <nav class="flex-1 w-full space-y-1 overflow-y-auto custom-scrollbar pr-2">
      <button 
        v-for="tab in [
          { id: 'dashboard', icon: LayoutDashboard, label: '仪表盘' },
          { id: 'groups', icon: Layers, label: '策略组' },
          { id: 'proxies', icon: Globe, label: '代理节点' },
          { id: 'connections', icon: Activity, label: '实时连接' },
          { id: 'rules', icon: ListFilter, label: '分流规则' },
          { id: 'logs', icon: Terminal, label: '运行日志' },
          { id: 'settings', icon: Settings, label: '系统设置' }
        ]"
        :key="tab.id"
        @click="setCurrentTab(tab.id)"
        :class="[
          'w-full flex items-center justify-center md:justify-start px-4 py-2.5 rounded-xl transition-all duration-300 group mb-1',
          proxyStore.currentTab === tab.id 
            ? themeStore.getCurrentTheme()?.bg + ' ' + themeStore.getCurrentTheme()?.text
            : 'text-gray-400 hover:bg-gray-50 hover:text-gray-600'
        ]"
      >
        <component :is="tab.icon" :class="['w-5 h-5 md:mr-3', proxyStore.currentTab === tab.id ? '' : 'group-hover:scale-110 transition-transform']" />
        <span class="font-medium hidden md:block text-sm">{{ tab.label }}</span>
      </button>
    </nav>

    <!-- Bottom Actions -->
    <div class="mt-auto w-full space-y-4 pt-4 border-t border-gray-50 shrink-0">
      <div class="bg-gray-50 rounded-2xl p-3 hidden md:block">
        <div class="flex items-center justify-between mb-2">
          <span class="text-xs text-gray-400 font-medium uppercase tracking-wider">内核状态</span>
          <div :class="['w-2 h-2 rounded-full', proxyStore.isConnected ? 'bg-green-400 animate-pulse' : 'bg-gray-300']"></div>
        </div>
        <p class="text-sm font-semibold text-gray-700 truncate">{{ proxyStore.selectedKernel }}</p>
      </div>
      
      <div class="flex items-center justify-between px-2">
        <button @click="toggleHelp" class="p-2 rounded-xl text-gray-400 hover:bg-gray-50 hover:text-emerald-500 transition-colors">
          <Info class="w-5 h-5" />
        </button>
        <button class="p-2 rounded-xl text-gray-400 hover:bg-gray-50 hover:text-gray-600 transition-colors">
          <Moon class="w-5 h-5" />
        </button>
      </div>
    </div>
  </aside>
</template>
