<script setup lang="ts">
import { ref, computed } from 'vue';
import { Trash2 } from 'lucide-vue-next';
import { useProxyStore } from '../../stores/proxyStore';

const proxyStore = useProxyStore();
const logFilter = ref<'all' | 'INFO' | 'WARN' | 'ERROR'>('all');

const filteredLogs = computed(() => {
  if (logFilter.value === 'all') return proxyStore.logs;
  return proxyStore.logs.filter(l => l.level === logFilter.value);
});

const setFilter = (level: 'all' | 'INFO' | 'WARN' | 'ERROR') => {
  logFilter.value = level;
};
</script>

<template>
  <div class="h-full flex flex-col">
    <div class="flex items-center justify-between mb-4">
      <div class="flex space-x-2">
        <button
          v-for="opt in [
            { key: 'all', label: '全部' },
            { key: 'ERROR', label: '错误' },
            { key: 'WARN', label: '警告' },
            { key: 'INFO', label: '信息' }
          ]"
          :key="opt.key"
          @click="setFilter(opt.key as any)"
          :class="[
            'px-3 py-1.5 rounded-xl text-xs font-bold transition-colors',
            logFilter === opt.key
              ? 'bg-gray-800 text-white'
              : 'bg-white text-gray-400 hover:bg-gray-50'
          ]"
        >
          {{ opt.label }}
        </button>
      </div>
      <button @click="proxyStore.logs = []" class="p-2 text-gray-400 hover:text-red-500">
        <Trash2 class="w-5 h-5" />
      </button>
    </div>

    <div class="flex-1 bg-gray-900 rounded-[2rem] p-6 font-mono text-xs overflow-y-auto custom-scrollbar shadow-inner">
      <div v-if="filteredLogs.length === 0" class="flex items-center justify-center h-full text-gray-600">
        <p>{{ proxyStore.logs.length === 0 ? '等待日志...' : '没有匹配的日志' }}</p>
      </div>
      <div v-for="(log, i) in filteredLogs" :key="i" class="mb-2 flex space-x-4">
        <span class="text-gray-600 shrink-0">{{ log.time }}</span>
        <span :class="[
          'font-bold shrink-0',
          log.level === 'INFO' ? 'text-emerald-400' : log.level === 'WARN' ? 'text-amber-400' : log.level === 'DEBUG' ? 'text-blue-400' : 'text-red-400'
        ]">{{ log.level }}</span>
        <span class="text-gray-300 break-all">{{ log.msg }}</span>
      </div>
    </div>
  </div>
</template>
