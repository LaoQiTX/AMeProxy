// mihomo API 基础 URL
const API_BASE = 'http://127.0.0.1:9090';
const API_SECRET = 'secret'; // 实际使用时应该从配置中读取

// 动态获取invoke函数
const getInvoke = async () => {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    // 确保invoke是一个函数
    if (typeof invoke === 'function') {
      return invoke;
    } else {
      console.warn('Invoke is not a function, using mock');
      return async (command: string, args?: any) => {
        console.log(`模拟调用Tauri命令: ${command}`, args);
        return Promise.resolve({});
      };
    }
  } catch (e) {
    console.warn('Tauri API not available, using mock');
    return async (command: string, args?: any) => {
      console.log(`模拟调用Tauri命令: ${command}`, args);
      return Promise.resolve({});
    };
  }
};

// 代理项类型
export interface ProxyItem {
  name: string
  type: string
  now?: boolean
  latency?: number
  history?: Array<{ delay: number, time: string }>
}

// 代理组类型
export interface ProxyGroup {
  name: string
  type: string
  all: string[]
  now: string
}

// API 请求工具
async function apiRequest<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
  const url = `${API_BASE}${endpoint}`;
  const headers = {
    'Authorization': `Bearer ${API_SECRET}`,
    'Content-Type': 'application/json',
    ...options.headers
  };

  try {
    const response = await fetch(url, {
      ...options,
      headers
    });

    if (!response.ok) {
      throw new Error(`API 请求失败: ${response.status} ${response.statusText}`);
    }

    return await response.json();
  } catch (error) {
    console.error(`API 请求错误 ${endpoint}:`, error);
    //  fallback 到 Tauri 命令
    const invoke = await getInvoke();
    return await invoke<T>(endpoint.replace(/\//g, '_').substring(1), options.body ? JSON.parse(options.body as string) : {});
  }
}

// 获取代理列表
export async function getProxies() {
  return await apiRequest<{ proxies: Record<string, any> }>('/proxies');
}

// 选择代理
export async function changeProxy(group: string, proxy: string) {
  return await apiRequest('/proxies/' + encodeURIComponent(group), {
    method: 'PUT',
    body: JSON.stringify({ name: proxy })
  });
}

// 测试代理延迟
export async function testProxy(proxy: string) {
  const response = await apiRequest<{ delay: number }>(`/proxies/${encodeURIComponent(proxy)}/delay`);
  return response.delay;
}

// 启动代理内核
export async function startCore() {
  const invoke = await getInvoke();
  return await invoke('start_core');
}

// 停止代理内核
export async function stopCore() {
  const invoke = await getInvoke();
  return await invoke('stop_core');
}

// 检查代理是否正在运行
export async function is_proxy_running() {
  const invoke = await getInvoke();
  return await invoke<boolean>('is_proxy_running');
}

// 获取代理提供者列表
export async function getProviders() {
  return await apiRequest<{ providers: Record<string, any> }>('/providers/proxies');
}

// 获取特定提供者的代理
export async function getProviderProxies(providerName: string) {
  return await apiRequest<any>(`/providers/proxies/${encodeURIComponent(providerName)}`);
}

// 触发提供者健康检查
export async function triggerProviderHealthCheck(providerName: string) {
  return await apiRequest(`/providers/proxies/${encodeURIComponent(providerName)}/healthcheck`);
}
