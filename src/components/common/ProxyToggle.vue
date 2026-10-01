<script setup lang="ts">
import { Power, LoaderCircle } from 'lucide-vue-next';
import { useProxyStore } from '../../stores/proxyStore';
const store = useProxyStore();
</script>

<template>
  <button class="proxy-toggle" :class="{ connected: store.systemProxyEnabled || store.tunMode }"
    @click="store.toggleConnection()" :disabled="store.isBusy || store.subscriptionBusy || store.previewMode"
    :aria-busy="store.isBusy">
    <LoaderCircle v-if="store.isBusy" :size="17" class="animate-spin" />
    <Power v-else :size="17" />
    <span>{{ store.isBusy ? '正在处理…' : store.systemProxyEnabled || store.tunMode ? '关闭代理' : store.recoveryPending ? '恢复系统设置' : '开启代理' }}</span>
  </button>
</template>
