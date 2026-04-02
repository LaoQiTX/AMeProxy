<script setup lang="ts">
import { ref } from 'vue'
import { Info, Plus } from 'lucide-vue-next'

// Components
import Sidebar from './components/common/Sidebar.vue'
import Header from './components/common/Header.vue'
import Dashboard from './components/dashboard/Dashboard.vue'
import ProxyGroups from './components/proxies/ProxyGroups.vue'
import ProxyNodes from './components/proxies/ProxyNodes.vue'
import Connections from './components/proxies/Connections.vue'
import Rules from './components/proxies/Rules.vue'
import Logs from './components/proxies/Logs.vue'
import Settings from './components/settings/Settings.vue'

// Stores
import { useThemeStore } from './stores/themeStore'
import { useProxyStore } from './stores/proxyStore'

const showHelp = ref(false)
const themeStore = useThemeStore()
const proxyStore = useProxyStore()

const toggleHelp = () => {
  showHelp.value = !showHelp.value
}
</script>

<template>
  <div :class="['h-screen w-screen flex transition-colors duration-500 overflow-hidden select-none', themeStore.getCurrentTheme()?.bg]">
    
    <!-- Sidebar -->
    <Sidebar />

    <!-- Main Content -->
    <main class="flex-1 h-full flex flex-col min-w-0 relative">
      <!-- Header -->
      <Header />

      <!-- Scrollable Area -->
      <div class="flex-1 overflow-y-auto p-8 custom-scrollbar bg-white/10">
        
        <!-- Dashboard Tab -->
        <Dashboard v-if="proxyStore.currentTab === 'dashboard'" />

        <!-- Proxy Groups Tab -->
        <ProxyGroups v-else-if="proxyStore.currentTab === 'groups'" />

        <!-- Proxies Tab -->
        <ProxyNodes v-else-if="proxyStore.currentTab === 'proxies'" />

        <!-- Connections Tab -->
        <Connections v-else-if="proxyStore.currentTab === 'connections'" />

        <!-- Rules Tab -->
        <Rules v-else-if="proxyStore.currentTab === 'rules'" />

        <!-- Logs Tab -->
        <Logs v-else-if="proxyStore.currentTab === 'logs'" />

        <!-- Settings Tab -->
        <Settings v-else-if="proxyStore.currentTab === 'settings'" />

      </div>
    </main>

    <!-- Help Modal Overlay -->
    <transition name="fade">
      <div v-if="showHelp" class="fixed inset-0 bg-black/20 backdrop-blur-sm z-50 flex items-center justify-center p-4" @click.self="showHelp = false">
        <transition name="scale">
          <div v-if="showHelp" class="bg-white rounded-[2.5rem] shadow-2xl w-full max-w-2xl max-h-[80vh] overflow-hidden flex flex-col">
            <div class="p-8 border-b border-gray-50 flex items-center justify-between shrink-0">
              <div class="flex items-center space-x-4">
                <div class="p-3 bg-emerald-50 rounded-2xl">
                  <Info class="w-6 h-6 text-emerald-500" />
                </div>
                <h3 class="text-2xl font-black text-gray-800">功能说明书</h3>
              </div>
              <button @click="showHelp = false" class="p-2 hover:bg-gray-100 rounded-full transition-colors">
                <Plus class="w-6 h-6 text-gray-400 rotate-45" />
              </button>
            </div>
            
            <div class="flex-1 overflow-y-auto p-8 space-y-8 custom-scrollbar">
              <div v-for="help in [
                { title: '仪表盘', desc: '实时监控网络状态、流量消耗及核心运行指标。' },
                { title: '策略组', desc: '灵活配置代理策略，支持手动选择、自动测速、故障转移等高级逻辑。' },
                { title: '代理节点', desc: '管理您的所有订阅和节点，支持 Shadowsocks, Vmess, Trojan 等主流协议。' },
                { title: '实时连接', desc: '查看每一个正在进行的网络请求，包括目标主机、所属进程及实时速率。' },
                { title: '分流规则', desc: '基于域名、关键词或地理位置自动决定流量去向，实现智能分流。' },
                { title: '运行日志', desc: '详细记录内核运行状态，是排查网络问题的得力助手。' },
                { title: 'TUN 模式', desc: '虚拟网卡模式，能够接管系统全局流量，让不带代理设置的应用也能轻松出海。' }
              ]" :key="help.title" class="flex items-start space-x-6">
                <div class="p-3 bg-gray-50 rounded-2xl shrink-0">
                  <Info class="w-6 h-6 text-gray-400" />
                </div>
                <div>
                  <h4 class="text-lg font-bold text-gray-800 mb-1">{{ help.title }}</h4>
                  <p class="text-sm text-gray-500 leading-relaxed">{{ help.desc }}</p>
                </div>
              </div>
            </div>

            <div class="p-8 bg-gray-50/50 border-t border-gray-50 text-center shrink-0">
              <p class="text-xs text-gray-400 font-medium">AMeProxy v1.0.0</p>
            </div>
          </div>
        </transition>
      </div>
    </transition>
  </div>
</template>

<style>
.fade-enter-active, .fade-leave-active { transition: opacity 0.3s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }

.scale-enter-active, .scale-leave-active { transition: all 0.4s cubic-bezier(0.34, 1.56, 0.64, 1); }
.scale-enter-from, .scale-leave-to { transform: scale(0.9) translateY(20px); opacity: 0; }
.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(0, 0, 0, 0.05);
  border-radius: 10px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(0, 0, 0, 0.1);
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}
.animate-pulse {
  animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}
</style>
