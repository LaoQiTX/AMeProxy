<script setup lang="ts">
import { ref } from 'vue';
import { ArrowRight, Pencil, X } from 'lucide-vue-next';
import { useProxyStore } from '../../stores/proxyStore';
import { useThemeStore } from '../../stores/themeStore';
import ProxyToggle from '../common/ProxyToggle.vue';
import type { Subscription } from '../../types';
const store = useProxyStore();
const themes = useThemeStore();
const editor = ref<HTMLDialogElement>();
const edit = ref({ oldName: '', name: '', url: '' });
function openEdit(sub: Subscription) {
  edit.value = { oldName: sub.name, name: sub.name, url: sub.url };
  editor.value?.showModal();
}
async function saveEdit() {
  try {
    await store.updateSubscription(edit.value.oldName, edit.value.name.trim(), edit.value.url.trim());
    editor.value?.close();
  } catch { /* Keep form and the visible store error. */ }
}
</script>

<template>
  <div class="settings-page">
    <section class="settings-section">
      <h2>代理与网络</h2>
      <div class="panel">
        <div class="setting-row">
          <div><h3>Windows 系统代理</h3><p>开启后自动启动内核并设置 HTTP / HTTPS 代理。<br>关闭代理或正常退出应用时，恢复之前的系统设置。</p></div>
          <ProxyToggle />
        </div>
        <div class="setting-row">
          <div><h3>TUN 模式 <span class="experimental-label">实验功能</span></h3><p>接管不遵循系统代理的应用，需要管理员权限及有效的 DNS / 路由配置。</p></div>
          <button class="switch" role="switch" aria-label="TUN 模式" :aria-checked="store.tunMode"
            :disabled="store.tunBusy || !store.isConnected || store.isBusy" @click="store.toggleTunMode()"><i></i></button>
        </div>
        <div class="setting-row"><div><h3>混合代理端口</h3><p>供手动设置 HTTP / SOCKS5 的应用使用</p></div><span class="setting-value">{{ store.mixedPort ? '127.0.0.1:' + store.mixedPort : '未配置' }}</span></div>
        <div class="setting-row"><div><h3>允许局域网连接</h3><p>由内核配置文件管理</p></div><span class="setting-value">{{ store.allowLan ? '已允许' : '仅本机' }}</span></div>
        <div class="setting-row"><div><h3>Mihomo 内核</h3><p>内核运行与系统代理启用是两个独立状态</p></div><span class="setting-value">{{ store.isConnected ? '运行中' : '已停止' }}</span></div>
      </div>
    </section>

    <section class="settings-section">
      <h2>外观</h2>
      <div class="panel">
        <div class="setting-row"><div><h3>强调色</h3><p>为工作空间选择一种颜色，自动保存在此设备。</p></div></div>
        <div class="theme-options">
          <button v-for="theme in themes.themes" :key="theme.id" class="theme-option" :aria-pressed="themes.currentTheme === theme.id" @click="themes.setTheme(theme.id)">
            <i :style="{ background: theme.color }"></i>{{ theme.name }}
          </button>
        </div>
      </div>
    </section>

    <section class="settings-section">
      <h2>订阅管理</h2>
      <div class="panel">
        <div class="setting-row"><div><h3>{{ store.subscriptions.length }} 个已保存订阅</h3><p>在节点页导入订阅、查看节点和执行测速。</p></div><button class="text-button" @click="store.setCurrentTab('proxies')">管理节点<ArrowRight :size="14" /></button></div>
        <div v-for="sub in store.subscriptions" :key="sub.name" class="setting-row">
          <div class="min-w-0"><h3 class="break-all">{{ sub.name }}</h3><p>{{ sub.name === store.activeSubscription ? '当前使用 · ' + sub.count + ' 个节点' : '未启用 · 已保存' }} · {{ sub.updateTime }}</p></div>
          <button class="text-button" :disabled="store.subscriptionBusy || store.isBusy" @click="openEdit(sub)"><Pencil :size="13" />编辑</button>
        </div>
      </div>
    </section>

    <dialog ref="editor" class="edit-dialog" aria-labelledby="edit-dialog-title" @cancel="store.subscriptionBusy && $event.preventDefault()">
      <form @submit.prevent="saveEdit">
        <div class="section-heading"><h2 id="edit-dialog-title">编辑订阅</h2><button type="button" class="icon-button" aria-label="关闭编辑" :disabled="store.subscriptionBusy" @click="editor?.close()"><X :size="18" /></button></div>
        <label class="form-label" for="edit-name">订阅名称</label>
        <input id="edit-name" v-model="edit.name" class="form-input" required :disabled="store.subscriptionBusy" />
        <label class="form-label" for="edit-url">Clash YAML 订阅链接</label>
        <input id="edit-url" v-model="edit.url" class="form-input" type="url" required :disabled="store.subscriptionBusy" />
        <p v-if="store.error" role="alert" class="form-error">{{ store.error }}</p>
        <div class="dialog-actions"><button type="button" class="secondary-button" :disabled="store.subscriptionBusy" @click="editor?.close()">取消</button><button type="submit" class="proxy-toggle" :disabled="store.subscriptionBusy || store.isBusy">{{ store.subscriptionBusy ? '保存中…' : '保存修改' }}</button></div>
      </form>
    </dialog>
  </div>
</template>
