<script setup lang="ts">
import { LayoutDashboard, Globe2, Settings2, Activity, Layers, ListFilter, Terminal, Cpu, ArrowUpRight } from 'lucide-vue-next';
import { useProxyStore } from '../../stores/proxyStore';
const store = useProxyStore();
const tabs = [
  { id: 'dashboard', icon: LayoutDashboard, label: '网络概览' },
  { id: 'groups', icon: Layers, label: '策略组' },
  { id: 'proxies', icon: Globe2, label: '订阅与节点' },
  { id: 'connections', icon: Activity, label: '实时连接' },
  { id: 'rules', icon: ListFilter, label: '分流规则' },
  { id: 'logs', icon: Terminal, label: '运行日志' },
];
</script>

<template>
  <aside class="sidebar">
    <button class="brand" @click="store.setCurrentTab('dashboard')" aria-label="AMeProxy 网络概览">
      <img src="/icon.png" alt="" width="38" height="38" />
      <span><strong>AMeProxy</strong><small>让连接更简单</small></span>
    </button>
    <div class="nav-caption">工作空间</div>
    <nav class="nav-list" aria-label="主导航">
      <button v-for="tab in tabs" :key="tab.id" @click="store.setCurrentTab(tab.id)"
        :class="['nav-item', { active: store.currentTab === tab.id }]"
        :aria-current="store.currentTab === tab.id ? 'page' : undefined" :title="tab.label">
        <component :is="tab.icon" :size="18" /><span>{{ tab.label }}</span>
        <span v-if="tab.id === 'connections' && store.connections.length" class="nav-count">{{ store.connections.length }}</span>
      </button>
    </nav>
    <div class="sidebar-bottom">
      <button class="nav-item" :class="{ active: store.currentTab === 'settings' }" @click="store.setCurrentTab('settings')"
        :aria-current="store.currentTab === 'settings' ? 'page' : undefined" title="偏好设置">
        <Settings2 :size="18" /><span>偏好设置</span>
      </button>
      <div class="kernel-card" :title="store.isConnected ? 'Mihomo 内核运行中' : 'Mihomo 内核未运行'">
        <div class="kernel-title"><Cpu :size="16" /><span>Mihomo</span><i class="status-dot" :class="{ online: store.isConnected }"></i></div>
        <p>{{ store.isConnected ? '内核运行中' : '内核未运行' }}<span>本地服务</span></p>
      </div>
      <div class="sidebar-footer"><span>AMeProxy · 1.0</span><ArrowUpRight :size="12" aria-hidden="true" /></div>
    </div>
  </aside>
</template>
