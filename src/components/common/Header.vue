<script setup lang="ts">
import { computed } from 'vue';
import { useProxyStore } from '../../stores/proxyStore';
import ProxyToggle from './ProxyToggle.vue';
const store = useProxyStore();
const pages: Record<string, { title: string; description: string }> = {
  dashboard: { title: '网络概览', description: '连接状态与实时网络活动' },
  groups: { title: '策略组', description: '选择流量出口，按需切换连接策略' },
  proxies: { title: '订阅与节点', description: '管理订阅，发现可用的连接节点' },
  connections: { title: '实时连接', description: '查看内核正在处理的网络请求' },
  rules: { title: '分流规则', description: '按照规则顺序决定流量去向' },
  logs: { title: '运行日志', description: '检查运行记录，定位连接问题' },
  settings: { title: '偏好设置', description: '网络选项与个性化外观' },
};
const page = computed(() => pages[store.currentTab] || pages.dashboard);
</script>

<template>
  <header class="page-header">
    <div class="page-heading"><h1>{{ page.title }}</h1><p>{{ page.description }}</p></div>
    <div class="header-actions">
      <ProxyToggle v-if="store.currentTab !== 'dashboard'" />
    </div>
  </header>
</template>
