<script setup lang="ts">
import { Layers } from 'lucide-vue-next';
import { useProxyStore } from '../../stores/proxyStore';

const proxyStore = useProxyStore();
</script>

<template>
  <div class="space-y-6">
    <div v-if="proxyStore.proxyGroups.length === 0" class="flex flex-col items-center justify-center py-20 text-gray-400">
      <Layers class="w-16 h-16 mb-4 text-gray-200" />
      <p class="text-lg font-bold">暂无策略组数据</p>
      <p class="text-sm mt-2">请先开启代理并确保内核已成功读取配置文件</p>
    </div>

    <div v-else v-for="group in proxyStore.proxyGroups" :key="group.name" class="bg-white rounded-3xl p-6 shadow-sm border border-gray-50">
      <div class="flex items-center justify-between mb-4">
        <div class="flex items-center space-x-3">
          <div class="p-2 bg-gray-50 rounded-xl">
            <Layers class="w-5 h-5 text-gray-400" />
          </div>
          <div>
            <h3 class="font-bold text-gray-800">{{ group.name }}</h3>
            <p class="text-xs text-gray-400 font-medium uppercase">{{ group.type }}</p>
          </div>
        </div>
        <span class="text-xs font-bold text-emerald-500 bg-emerald-50 px-2 py-1 rounded-lg">{{ group.selected }}</span>
      </div>
      <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-3">
        <button 
          v-for="opt in group.options" 
          :key="opt"
          @click="proxyStore.switchProxy(group.name, opt)"
          :class="[
            'px-4 py-2 rounded-xl text-sm font-medium transition-all',
            group.selected === opt ? 'bg-gray-800 text-white shadow-md' : 'bg-gray-50 text-gray-500 hover:bg-gray-100'
          ]"
        >
          {{ opt }}
        </button>
      </div>
    </div>
  </div>
</template>
