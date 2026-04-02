<script setup lang="ts">
import { Trash2 } from 'lucide-vue-next';
import { useProxyStore } from '../../stores/proxyStore';

const proxyStore = useProxyStore();
</script>

<template>
  <div class="h-full flex flex-col">
    <div class="flex items-center justify-between mb-4">
      <div class="flex space-x-2">
        <button class="px-3 py-1.5 bg-gray-800 text-white rounded-xl text-xs font-bold">全部</button>
        <button class="px-3 py-1.5 bg-white text-gray-400 rounded-xl text-xs font-bold">错误</button>
        <button class="px-3 py-1.5 bg-white text-gray-400 rounded-xl text-xs font-bold">警告</button>
      </div>
      <button @click="proxyStore.logs = []" class="p-2 text-gray-400 hover:text-red-500"><Trash2 class="w-5 h-5" /></button>
    </div>
    <div class="flex-1 bg-gray-900 rounded-[2rem] p-6 font-mono text-xs overflow-y-auto custom-scrollbar shadow-inner">
      <div v-for="log in proxyStore.logs" :key="log.time" class="mb-2 flex space-x-4">
        <span class="text-gray-600">{{ log.time }}</span>
        <span :class="[
          'font-bold',
          log.level === 'INFO' ? 'text-emerald-400' : log.level === 'WARN' ? 'text-amber-400' : log.level === 'DEBUG' ? 'text-blue-400' : 'text-red-400'
        ]">{{ log.level }}</span>
        <span class="text-gray-300">{{ log.msg }}</span>
      </div>
    </div>
  </div>
</template>
