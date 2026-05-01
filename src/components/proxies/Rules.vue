<script setup lang="ts">
import { ref } from 'vue';
import { Trash2, Plus } from 'lucide-vue-next';
import { useProxyStore } from '../../stores/proxyStore';

const proxyStore = useProxyStore();
const showAddModal = ref(false);
const newRule = ref({ type: 'MATCH', payload: '', strategy: '' });

const removeRule = (index: number) => {
  proxyStore.rules.splice(index, 1);
};

const addRule = () => {
  if (!newRule.value.payload && newRule.value.type !== 'MATCH') return;
  proxyStore.rules.push({
    type: newRule.value.type,
    payload: newRule.value.payload || '*',
    strategy: newRule.value.strategy || '默认'
  });
  newRule.value = { type: 'MATCH', payload: '', strategy: '' };
  showAddModal.value = false;
};
</script>

<template>
  <div class="space-y-4">
    <div class="flex items-center justify-between mb-6">
      <h3 class="text-sm font-bold text-gray-400 uppercase tracking-widest">分流规则列表 ({{ proxyStore.rules.length }})</h3>
      <button @click="showAddModal = true" class="px-4 py-2 bg-emerald-500 text-white rounded-2xl text-sm font-bold shadow-lg shadow-emerald-100 flex items-center space-x-2 hover:bg-emerald-600 transition-colors">
        <Plus class="w-4 h-4" />
        <span>添加规则</span>
      </button>
    </div>

    <div v-if="proxyStore.rules.length === 0" class="flex flex-col items-center justify-center py-20 text-gray-400 bg-white/50 rounded-3xl border border-gray-50 border-dashed">
      <p class="text-sm font-bold">暂无分流规则</p>
      <p class="text-xs mt-1">请开启代理并确保配置已加载</p>
    </div>

    <div v-else class="grid grid-cols-1 gap-3">
      <div v-for="(rule, index) in proxyStore.rules" :key="index" class="bg-white p-4 rounded-2xl shadow-sm border border-gray-50 flex items-center justify-between group hover:shadow-md transition-shadow">
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
          <button @click="removeRule(index)" class="p-2 opacity-0 group-hover:opacity-100 transition-all text-gray-300 hover:text-red-400">
            <Trash2 class="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>

    <!-- 添加规则弹窗 -->
    <Transition name="fade">
      <div v-if="showAddModal" class="fixed inset-0 bg-black/20 backdrop-blur-sm z-50 flex items-center justify-center p-4" @click.self="showAddModal = false">
        <div class="bg-white rounded-[2.5rem] shadow-2xl w-full max-w-md p-8">
          <h3 class="text-2xl font-black text-gray-800 mb-6">添加分流规则</h3>
          <div class="space-y-4">
            <div>
              <label class="text-xs font-bold text-gray-400 uppercase mb-2 block">规则类型</label>
              <select v-model="newRule.type" class="w-full bg-gray-50 border-none rounded-2xl px-5 py-3 text-sm focus:ring-2 focus:ring-emerald-400 outline-none">
                <option value="MATCH">MATCH（全匹配）</option>
                <option value="DOMAIN">DOMAIN（域名）</option>
                <option value="DOMAIN-SUFFIX">DOMAIN-SUFFIX（域名后缀）</option>
                <option value="DOMAIN-KEYWORD">DOMAIN-KEYWORD（域名关键词）</option>
                <option value="IP-CIDR">IP-CIDR（IP段）</option>
                <option value="GEOIP">GEOIP（地理位置）</option>
                <option value="GEOSITE">GEOSITE（网站分类）</option>
              </select>
            </div>
            <div v-if="newRule.type !== 'MATCH'">
              <label class="text-xs font-bold text-gray-400 uppercase mb-2 block">规则内容</label>
              <input v-model="newRule.payload" type="text" placeholder="例如: google.com" class="w-full bg-gray-50 border-none rounded-2xl px-5 py-3 text-sm focus:ring-2 focus:ring-emerald-400 outline-none" />
            </div>
            <div>
              <label class="text-xs font-bold text-gray-400 uppercase mb-2 block">策略组</label>
              <select v-model="newRule.strategy" class="w-full bg-gray-50 border-none rounded-2xl px-5 py-3 text-sm focus:ring-2 focus:ring-emerald-400 outline-none">
                <option v-for="g in proxyStore.proxyGroups" :key="g.name" :value="g.name">{{ g.name }}</option>
                <option value="默认">默认</option>
              </select>
            </div>
          </div>
          <div class="mt-8 flex space-x-4">
            <button @click="showAddModal = false" class="flex-1 py-3 rounded-2xl font-bold text-gray-400 hover:bg-gray-50 transition-colors">取消</button>
            <button @click="addRule" class="flex-1 py-3 rounded-2xl font-bold text-white shadow-lg bg-emerald-500 hover:bg-emerald-600 transition-colors">确认添加</button>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
