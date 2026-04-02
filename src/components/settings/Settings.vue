<script setup lang="ts">
import { useProxyStore } from '../../stores/proxyStore';
import { ref } from 'vue';

const proxyStore = useProxyStore();

const toggleTunMode = () => {
  proxyStore.toggleTunMode();
};

const newSubscription = ref({
  name: '',
  url: ''
});

const addSubscription = () => {
  if (newSubscription.value.name && newSubscription.value.url) {
    proxyStore.addSubscription(newSubscription.value.name, newSubscription.value.url);
    newSubscription.value = { name: '', url: '' };
  }
};

const editSubscription = ref({
  name: '',
  url: '',
  originalName: ''
});

const showEditModal = ref(false);

const openEditModal = (sub: any) => {
  editSubscription.value = {
    name: sub.name,
    url: sub.url,
    originalName: sub.name
  };
  showEditModal.value = true;
};

const saveEdit = () => {
  if (editSubscription.value.name && editSubscription.value.url) {
    proxyStore.updateSubscription(editSubscription.value.originalName, editSubscription.value.name, editSubscription.value.url);
    showEditModal.value = false;
  }
};

const removeSubscription = (name: string) => {
  if (confirm('确定要删除这个订阅吗？')) {
    proxyStore.removeSubscription(name);
  }
};
</script>

<template>
  <div class="max-w-2xl space-y-8">
    <section>
      <h3 class="text-sm font-bold text-gray-400 uppercase tracking-widest mb-4">核心设置</h3>
      <div class="bg-white rounded-3xl shadow-sm border border-gray-50 divide-y divide-gray-50 overflow-hidden">
        <div class="p-6 flex items-center justify-between">
          <div>
            <p class="font-bold text-gray-800">内核切换</p>
            <p class="text-xs text-gray-400 mt-1">选择您偏好的代理内核</p>
          </div>
          <select v-model="proxyStore.selectedKernel" class="bg-gray-50 border-none rounded-xl px-4 py-2 text-sm font-bold text-gray-700 focus:ring-2 focus:ring-emerald-400 outline-none">
            <option v-for="k in proxyStore.kernels" :key="k" :value="k">{{ k }}</option>
          </select>
        </div>
        <div class="p-6 flex items-center justify-between">
          <div>
            <p class="font-bold text-gray-800">TUN 模式</p>
            <p class="text-xs text-gray-400 mt-1">接管系统流量，支持所有应用</p>
          </div>
          <div 
            @click="toggleTunMode"
            :class="['w-12 h-6 rounded-full relative cursor-pointer transition-colors duration-300', proxyStore.tunMode ? 'bg-emerald-500' : 'bg-gray-200']"
          >
            <div :class="['absolute top-1 w-4 h-4 bg-white rounded-full shadow-sm transition-all duration-300', proxyStore.tunMode ? 'right-1' : 'left-1']"></div>
          </div>
        </div>
        <div class="p-6 flex items-center justify-between">
          <div>
            <p class="font-bold text-gray-800">开机自启</p>
            <p class="text-xs text-gray-400 mt-1">系统启动时自动运行 CuteProxy</p>
          </div>
          <div class="w-12 h-6 bg-emerald-500 rounded-full relative cursor-pointer">
            <div class="absolute right-1 top-1 w-4 h-4 bg-white rounded-full shadow-sm"></div>
          </div>
        </div>
      </div>
    </section>

    <section>
      <h3 class="text-sm font-bold text-gray-400 uppercase tracking-widest mb-4">网络设置</h3>
      <div class="bg-white rounded-3xl shadow-sm border border-gray-50 divide-y divide-gray-50 overflow-hidden">
        <div class="p-6 flex items-center justify-between">
          <div>
            <p class="font-bold text-gray-800">混合端口</p>
            <p class="text-xs text-gray-400 mt-1">HTTP/SOCKS5 代理端口</p>
          </div>
          <input type="text" value="7890" class="w-20 bg-gray-50 border-none rounded-xl px-4 py-2 text-sm font-bold text-gray-700 text-center" />
        </div>
        <div class="p-6 flex items-center justify-between">
          <div>
            <p class="font-bold text-gray-800">允许局域网连接</p>
            <p class="text-xs text-gray-400 mt-1">允许其他设备通过此代理上网</p>
          </div>
          <div class="w-12 h-6 bg-gray-200 rounded-full relative cursor-pointer">
            <div class="absolute left-1 top-1 w-4 h-4 bg-white rounded-full shadow-sm"></div>
          </div>
        </div>
      </div>
    </section>

    <section>
      <h3 class="text-sm font-bold text-gray-400 uppercase tracking-widest mb-4">订阅管理</h3>
      <div class="bg-white rounded-3xl shadow-sm border border-gray-50 divide-y divide-gray-50 overflow-hidden">
        <!-- 添加订阅 -->
        <div class="p-6">
          <h4 class="font-bold text-gray-700 mb-4">添加订阅</h4>
          <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
            <input 
              v-model="newSubscription.name"
              type="text" 
              placeholder="订阅名称" 
              class="bg-gray-50 border-none rounded-xl px-4 py-2 text-sm font-bold text-gray-700 focus:ring-2 focus:ring-emerald-400 outline-none"
            />
            <input 
              v-model="newSubscription.url"
              type="text" 
              placeholder="订阅链接"
              class="bg-gray-50 border-none rounded-xl px-4 py-2 text-sm font-bold text-gray-700 focus:ring-2 focus:ring-emerald-400 outline-none md:col-span-2"
            />
            <button 
              @click="addSubscription"
              class="bg-emerald-500 hover:bg-emerald-600 text-white font-bold py-2 px-4 rounded-xl transition-colors duration-300 md:col-span-3"
            >
              添加订阅
            </button>
          </div>
        </div>

        <!-- 订阅列表 -->
        <div class="divide-y divide-gray-50">
          <div 
            v-for="sub in proxyStore.subscriptions" 
            :key="sub.name"
            class="p-6 flex flex-col md:flex-row md:items-center md:justify-between"
          >
            <div class="mb-4 md:mb-0">
              <p class="font-bold text-gray-800">{{ sub.name }}</p>
              <p class="text-xs text-gray-400 mt-1">{{ sub.url }}</p>
              <p class="text-xs text-gray-400 mt-1">更新时间: {{ sub.updateTime }}</p>
              <p class="text-xs text-gray-400 mt-1">节点数量: {{ sub.count }}</p>
            </div>
            <div class="flex space-x-2">
              <button 
                @click="openEditModal(sub)"
                class="bg-blue-500 hover:bg-blue-600 text-white font-bold py-2 px-4 rounded-xl text-sm transition-colors duration-300"
              >
                编辑
              </button>
              <button 
                @click="removeSubscription(sub.name)"
                class="bg-red-500 hover:bg-red-600 text-white font-bold py-2 px-4 rounded-xl text-sm transition-colors duration-300"
              >
                删除
              </button>
            </div>
          </div>
        </div>

        <!-- 无订阅提示 -->
        <div v-if="proxyStore.subscriptions.length === 0" class="p-6 text-center">
          <p class="text-gray-400">暂无订阅，请添加订阅</p>
        </div>
      </div>
    </section>

    <!-- 编辑订阅模态框 -->
    <div v-if="showEditModal" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div class="bg-white rounded-3xl p-6 max-w-md w-full">
        <h4 class="font-bold text-gray-800 mb-4">编辑订阅</h4>
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-bold text-gray-600 mb-2">订阅名称</label>
            <input 
              v-model="editSubscription.name"
              type="text" 
              class="bg-gray-50 border-none rounded-xl px-4 py-2 text-sm font-bold text-gray-700 focus:ring-2 focus:ring-emerald-400 outline-none w-full"
            />
          </div>
          <div>
            <label class="block text-sm font-bold text-gray-600 mb-2">订阅链接</label>
            <input 
              v-model="editSubscription.url"
              type="text" 
              class="bg-gray-50 border-none rounded-xl px-4 py-2 text-sm font-bold text-gray-700 focus:ring-2 focus:ring-emerald-400 outline-none w-full"
            />
          </div>
          <div class="flex space-x-2">
            <button 
              @click="saveEdit"
              class="bg-emerald-500 hover:bg-emerald-600 text-white font-bold py-2 px-4 rounded-xl transition-colors duration-300 flex-1"
            >
              保存
            </button>
            <button 
              @click="showEditModal = false"
              class="bg-gray-500 hover:bg-gray-600 text-white font-bold py-2 px-4 rounded-xl transition-colors duration-300 flex-1"
            >
              取消
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
