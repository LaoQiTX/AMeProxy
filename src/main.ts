import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import './index.css'
import { useProxyStore } from './stores/proxyStore'

const app = createApp(App)
const pinia = createPinia()
app.use(pinia)

// 等待下一个 tick，确保 Pinia 已完全初始化
setTimeout(() => {
  // 初始化应用
  const proxyStore = useProxyStore()
  proxyStore.initialize()
}, 0)

app.mount('#app')
