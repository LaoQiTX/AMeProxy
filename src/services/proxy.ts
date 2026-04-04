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
}

// 获取代理列表
export async function getProxies() {
  const invoke = await getInvoke();
  return await invoke<Record<string, any>>('get_proxies')
}

// 切换代理
export async function changeProxy(group: string, proxy: string) {
  const invoke = await getInvoke();
  return await invoke('change_proxy', { group, proxy })
}

// 测试代理延迟
export async function testProxy(proxy: string) {
  const invoke = await getInvoke();
  return await invoke<number>('test_proxy', { proxy })
}

// 启动代理内核
export async function startCore() {
  const invoke = await getInvoke();
  return await invoke('start_core')
}

// 停止代理内核
export async function stopCore() {
  const invoke = await getInvoke();
  return await invoke('stop_core')
}

// 检查代理是否正在运行
export async function is_proxy_running() {
  const invoke = await getInvoke();
  return await invoke<boolean>('is_proxy_running')
}

// 获取代理提供者列表
export async function getProviders() {
  const invoke = await getInvoke();
  return await invoke<Record<string, any>>('get_providers')
}
