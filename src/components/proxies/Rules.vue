<script setup lang="ts">
import { Trash2 } from 'lucide-vue-next';
import { useProxyStore } from '../../stores/proxyStore';

const proxyStore = useProxyStore();
</script>

<template>
  <div class="space-y-4">
    <div class="flex items-center justify-between mb-6">
      <h3 class="text-sm font-bold text-gray-400 uppercase tracking-widest">分流规则列表</h3>
      <button class="px-4 py-2 bg-emerald-500 text-white rounded-2xl text-sm font-bold shadow-lg shadow-emerald-100">添加规则</button>
    </div>

    <div v-if="proxyStore.rules.length === 0" class="flex flex-col items-center justify-center py-20 text-gray-400 bg-white/50 rounded-3xl border border-gray-50 border-dashed">
      <p class="text-sm font-bold">暂无分流规则</p>
      <p class="text-xs mt-1">请开启代理并确保配置已加载</p>
    </div>

    <div v-else class="grid grid-cols-1 gap-3">
      <div v-for="(rule, index) in proxyStore.rules" :key="index" class="bg-white p-4 rounded-2xl shadow-sm border border-gray-50 flex items-center justify-between group">
        <div class="flex items-center space-x-4">
          <div class="px-3 py-1 bg-gray-100 rounded-lg text-[10px] font-black text-gray-500 uppercase tracking-tighter">
            {{ rule.type }}
          </div>
          <span class="text-sm font-bold text-gray-700">{{ rule.payload }}</span>
        </div>
        <div class="flex items-center space-x-4">
          <span :class="['px-3 py-1 rounded-xl text-xs font-bold', rule.strategy === 'Proxy' ? 'bg-blue-50 text-blue-600' : 'bg-emerald-50 text-emerald-600']">
            {{ rule.strategy }}
          </span>
          <button class="p-2 opacity-0 group-hover:opacity-100 transition-opacity text-gray-300 hover:text-red-400">
            <Trash2 class="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
