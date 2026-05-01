<script setup lang="ts">
import { ref, computed } from 'vue';
import { Search, Trash2, Activity } from 'lucide-vue-next';
import { useProxyStore } from '../../stores/proxyStore';

const proxyStore = useProxyStore();
const searchQuery = ref('');

const filteredConnections = computed(() => {
  if (!searchQuery.value.trim()) return proxyStore.connections;
  const q = searchQuery.value.toLowerCase();
  return proxyStore.connections.filter(c =>
    c.host?.toLowerCase().includes(q) ||
    c.ip?.toLowerCase().includes(q) ||
    c.process?.toLowerCase().includes(q) ||
    c.rule?.toLowerCase().includes(q) ||
    c.group?.toLowerCase().includes(q)
  );
});

const handleCloseAll = () => {
  proxyStore.closeAllConnections();
};
</script>

<template>
  <div class="space-y-4">
    <div class="flex items-center justify-between mb-6">
      <div class="relative flex-1 max-w-md">
        <Search class="absolute left-4 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
        <input
          v-model="searchQuery"
          type="text"
          placeholder="搜索连接..."
          class="w-full bg-white border border-gray-100 rounded-2xl pl-11 pr-4 py-2.5 text-sm focus:ring-2 focus:ring-emerald-400 outline-none"
        />
      </div>
      <button
        @click="handleCloseAll"
        class="px-4 py-2.5 bg-red-50 text-red-500 rounded-2xl text-sm font-bold flex items-center space-x-2 hover:bg-red-100 transition-colors"
      >
        <Trash2 class="w-4 h-4" />
        <span>断开全部</span>
      </button>
    </div>

    <!-- 空状态 -->
    <div v-if="proxyStore.connections.length === 0" class="flex flex-col items-center justify-center py-20 text-gray-400 bg-white/50 rounded-3xl border border-gray-50 border-dashed">
      <Activity class="w-12 h-12 mb-2 text-gray-200" />
      <p class="text-sm font-bold">暂无连接</p>
      <p class="text-xs mt-1">当前没有活跃的网络连接</p>
    </div>

    <div v-else class="bg-white rounded-3xl shadow-sm border border-gray-50 overflow-hidden">
      <table class="w-full text-left border-collapse">
        <thead>
          <tr class="bg-gray-50/50 border-b border-gray-50">
            <th class="px-6 py-4 text-xs font-bold text-gray-400 uppercase">主机 / IP</th>
            <th class="px-6 py-4 text-xs font-bold text-gray-400 uppercase">进程</th>
            <th class="px-6 py-4 text-xs font-bold text-gray-400 uppercase">规则 / 策略组</th>
            <th class="px-6 py-4 text-xs font-bold text-gray-400 uppercase text-right">速度 / 时间</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-50">
          <tr v-for="conn in filteredConnections" :key="conn.id" class="hover:bg-gray-50/50 transition-colors group">
            <td class="px-6 py-4">
              <p class="text-sm font-bold text-gray-700">{{ conn.host }}</p>
              <p class="text-[10px] text-gray-400 font-medium">{{ conn.ip }}</p>
            </td>
            <td class="px-6 py-4">
              <span class="px-2 py-1 bg-gray-100 rounded-lg text-[10px] font-bold text-gray-500">{{ conn.process }}</span>
            </td>
            <td class="px-6 py-4">
              <div class="flex items-center space-x-2">
                <span :class="['px-2 py-0.5 rounded text-[10px] font-bold uppercase', conn.rule === 'Proxy' ? 'bg-blue-50 text-blue-600' : 'bg-gray-100 text-gray-500']">{{ conn.rule }}</span>
                <span class="text-xs text-gray-400">→ {{ conn.group }}</span>
              </div>
            </td>
            <td class="px-6 py-4 text-right">
              <p class="text-sm font-bold text-emerald-500">{{ conn.speed }}</p>
              <p class="text-[10px] text-gray-400">{{ conn.time }}</p>
            </td>
          </tr>
        </tbody>
      </table>

      <!-- 搜索结果为空 -->
      <div v-if="filteredConnections.length === 0 && searchQuery" class="p-8 text-center text-gray-400">
        <p class="text-sm">没有匹配的连接</p>
      </div>
    </div>
  </div>
</template>

