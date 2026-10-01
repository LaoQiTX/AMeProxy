<script setup lang="ts">
import { ref } from 'vue';
import { Trash2, Plus } from 'lucide-vue-next';
import { useProxyStore } from '../../stores/proxyStore';
import { setRules } from '../../services/proxy';
import type { Rule } from '../../types';

const proxyStore = useProxyStore();
const ruleDialog = ref<HTMLDialogElement>();
const saving = ref(false);
const newRule = ref({ type: 'DOMAIN-SUFFIX', payload: '', strategy: '' });

const persistRules = async (rules: Rule[]) => {
  if (saving.value) return false;
  saving.value = true;
  try {
    await setRules(rules.map(r => r.raw || (r.type === 'MATCH' ? r.type + ',' + r.strategy : [r.type, r.payload, r.strategy].join(','))));
    await proxyStore.fetchRules();
    return true;
  } catch (err) {
    proxyStore.error = '规则保存失败: ' + String(err);
    return false;
  } finally { saving.value = false; }
};

const removeRule = async (index: number) => {
  await persistRules(proxyStore.rules.filter((_, i) => i !== index));
};

const addRule = async () => {
  if (saving.value || (!newRule.value.payload && newRule.value.type !== 'MATCH')) return;
  const rules = [...proxyStore.rules];
  const terminal = rules.findIndex(r => r.type === 'MATCH');
  if (newRule.value.type === 'MATCH' && terminal >= 0) {
    proxyStore.error = '已经存在 MATCH 兜底规则，请先删除原规则';
    return;
  }
  const rule = { ...newRule.value, strategy: newRule.value.strategy || proxyStore.proxyGroups.find(g => g.name === '默认')?.name || 'DIRECT' };
  rules.splice(terminal < 0 ? rules.length : terminal, 0, rule);
  if (await persistRules(rules)) {
    newRule.value = { type: 'DOMAIN-SUFFIX', payload: '', strategy: '' };
    ruleDialog.value?.close();
  }
};
</script>

<template>
  <div class="space-y-4">
    <div class="flex items-center justify-between mb-6">
      <h3 class="text-sm font-semibold text-gray-500 uppercase tracking-widest">分流规则列表 ({{ proxyStore.rules.length }})</h3>
      <button @click="ruleDialog?.showModal()" class="px-4 py-2 bg-[var(--accent)] text-white rounded-lg text-sm font-semibold shadow-none flex items-center space-x-2 hover:opacity-90 transition-colors">
        <Plus class="w-4 h-4" />
        <span>添加规则</span>
      </button>
    </div>

    <div v-if="proxyStore.rules.length === 0" class="flex flex-col items-center justify-center py-20 text-gray-500 bg-white/50 rounded-xl border border-[var(--line)] border-dashed">
      <p class="text-sm font-semibold">暂无分流规则</p>
      <p class="text-xs mt-1">请开启代理并确保配置已加载</p>
    </div>

    <div v-else class="grid grid-cols-1 gap-3">
      <div v-for="(rule, index) in proxyStore.rules" :key="index" class="bg-white p-4 rounded-lg shadow-none border border-[var(--line)] flex items-center justify-between group hover:border-[var(--accent-border)] transition-shadow">
        <div class="flex flex-wrap items-center gap-3 min-w-0">
          <div class="px-3 py-1 bg-gray-100 rounded-lg text-[10px] font-semibold text-gray-500 uppercase tracking-tighter">
            {{ rule.type }}
          </div>
          <span class="text-sm font-semibold text-gray-700 break-all">{{ rule.payload }}</span>
        </div>
        <div class="flex items-center space-x-4">
          <span :class="['px-3 py-1 rounded-xl text-xs font-semibold', rule.strategy === 'Proxy' ? 'bg-blue-50 text-blue-600' : 'bg-[var(--accent-soft)] text-[var(--accent)]']">
            {{ rule.strategy }}
          </span>
          <button @click="removeRule(index)" :aria-label="'删除规则 ' + rule.type + ' ' + rule.payload" :disabled="saving" class="p-2 opacity-70 hover:opacity-100 transition-opacity text-gray-300 hover:text-red-400">
            <Trash2 class="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>

    <!-- 添加规则弹窗 -->
    <dialog ref="ruleDialog" class="edit-dialog" aria-labelledby="rule-dialog-title" @cancel="saving && $event.preventDefault()">
        <form @submit.prevent="addRule">
          <h3 id="rule-dialog-title" class="text-lg font-semibold text-gray-800 mb-6">添加分流规则</h3>
          <div class="space-y-4">
            <div>
              <label for="rule-type" class="form-label">规则类型</label>
              <select id="rule-type" v-model="newRule.type" class="w-full bg-gray-50 border-none rounded-lg px-5 py-3 text-sm focus:ring-2 focus:ring-[var(--accent)] outline-none">
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
              <label for="rule-payload" class="form-label">规则内容</label>
              <input id="rule-payload" required v-model="newRule.payload" type="text" placeholder="例如: google.com" class="w-full bg-gray-50 border-none rounded-lg px-5 py-3 text-sm focus:ring-2 focus:ring-[var(--accent)] outline-none" />
            </div>
            <div>
              <label for="rule-strategy" class="form-label">策略组</label>
              <select id="rule-strategy" v-model="newRule.strategy" class="w-full bg-gray-50 border-none rounded-lg px-5 py-3 text-sm focus:ring-2 focus:ring-[var(--accent)] outline-none">
                <option v-for="g in proxyStore.proxyGroups" :key="g.name" :value="g.name">{{ g.name }}</option>
                <option value="DIRECT">DIRECT（直连）</option>
              </select>
            </div>
          </div>
          <p v-if="proxyStore.error" class="form-error" role="alert">{{ proxyStore.error }}</p>
          <div class="mt-8 flex space-x-4">
            <button type="button" @click="ruleDialog?.close()" :disabled="saving" class="flex-1 py-3 rounded-lg font-semibold text-gray-500 hover:bg-gray-50 transition-colors">取消</button>
            <button type="submit" :disabled="saving || proxyStore.previewMode" class="flex-1 py-3 rounded-lg font-semibold text-white shadow-lg bg-[var(--accent)] hover:opacity-90 transition-colors">确认添加</button>
          </div>
        </form>
    </dialog>
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
