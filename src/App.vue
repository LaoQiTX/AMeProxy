<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { AlertCircle, X, Monitor } from 'lucide-vue-next';
import Sidebar from './components/common/Sidebar.vue';
import Header from './components/common/Header.vue';
import Dashboard from './components/dashboard/Dashboard.vue';
import ProxyGroups from './components/proxies/ProxyGroups.vue';
import ProxyNodes from './components/proxies/ProxyNodes.vue';
import Connections from './components/proxies/Connections.vue';
import Rules from './components/proxies/Rules.vue';
import Logs from './components/proxies/Logs.vue';
import Settings from './components/settings/Settings.vue';
import { useThemeStore } from './stores/themeStore';
import { useProxyStore } from './stores/proxyStore';
import { isDesktop } from './services/proxy';

const themeStore = useThemeStore();
const proxyStore = useProxyStore();
let unlisten: UnlistenFn[] = [];
let disposed = false;
onMounted(async () => {
  if (isDesktop()) {
    const stops = await Promise.all([
      listen<string>('proxy-shutdown-error', event => { proxyStore.error = event.payload; }),
      listen('proxy-status-changed', async () => {
        try {
          const status = await proxyStore.refreshProxyStatus();
          if (!status.kernelRunning) proxyStore.resetSession();
        } catch (error) { proxyStore.error = String(error); }
      }),
    ]);
    if (disposed) stops.forEach(stop => stop()); else unlisten = stops;
  }
});
onUnmounted(() => { disposed = true; unlisten.forEach(stop => stop()); proxyStore.stopPolling(); });
</script>

<template>
  <div class="app-shell" :data-theme="themeStore.currentTheme">
    <Sidebar />
    <main class="workspace">
      <Header />
      <div v-if="proxyStore.previewMode" class="preview-notice">
        <Monitor :size="15" /><span>界面预览 · 请在桌面应用中开启系统代理</span>
      </div>
      <div v-if="proxyStore.error" role="alert" class="error-notice">
        <AlertCircle :size="18" class="shrink-0" />
        <span>{{ proxyStore.error }}</span>
        <button @click="proxyStore.error = ''" class="icon-button" aria-label="关闭错误提示"><X :size="16" /></button>
      </div>
      <div class="page-content custom-scrollbar">
        <Dashboard v-if="proxyStore.currentTab === 'dashboard'" />
        <ProxyGroups v-else-if="proxyStore.currentTab === 'groups'" />
        <ProxyNodes v-else-if="proxyStore.currentTab === 'proxies'" />
        <Connections v-else-if="proxyStore.currentTab === 'connections'" />
        <Rules v-else-if="proxyStore.currentTab === 'rules'" />
        <Logs v-else-if="proxyStore.currentTab === 'logs'" />
        <Settings v-else-if="proxyStore.currentTab === 'settings'" />
      </div>
    </main>
  </div>
</template>
