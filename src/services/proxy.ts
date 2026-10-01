import { invoke, isTauri } from '@tauri-apps/api/core';
export const isDesktop = () => isTauri();

export function desktopInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) return Promise.reject(new Error('请使用桌面应用（npm run tauri:dev），浏览器预览无法控制内核'));
  return invoke<T>(command, args);
}

export interface ProxyItem {
  name: string;
  type: string;
  now?: string;
  history?: Array<{ delay: number; time: string }>;
}
export interface ProxyGroup { name: string; type: string; all: string[]; now: string }
export interface Provider {
  vehicleType?: string;
  proxies?: ProxyItem[];
  updatedAt?: string;
}
export interface ControllerConfig { wsUrl: string; secret: string }

export const getProxies = () => desktopInvoke<{ proxies: Record<string, ProxyItem & { all?: string[] }> }>('get_proxies');
export const changeProxy = (group: string, proxy: string) => desktopInvoke<void>('change_proxy', { group, proxy });
export const setRules = (rules: string[]) => desktopInvoke<void>('set_rules', { rules });
export const testProxy = (proxy: string) => desktopInvoke<number>('test_proxy', { proxy });
export const startCore = () => desktopInvoke<void>('start_core');
export const stopCore = () => desktopInvoke<void>('stop_core');
export interface ProxyStatus { kernelRunning: boolean; systemProxyEnabled: boolean; recoveryPending: boolean; error: string }
export const startProxy = () => desktopInvoke<void>('start_proxy');
export const stopProxy = () => desktopInvoke<void>('stop_proxy');
export const getProxyStatus = () => desktopInvoke<ProxyStatus>('get_proxy_status');
export type ProxyMode = 'rule' | 'global' | 'direct';
export const getProxyMode = () => desktopInvoke<ProxyMode>('get_proxy_mode');
export const setProxyMode = (mode: ProxyMode) => desktopInvoke<ProxyMode>('set_proxy_mode', { mode });
export interface DiagnosticCheck {
  key: 'kernel' | 'config' | 'systemProxy' | 'port' | 'dns' | 'target';
  state: 'success' | 'failed' | 'untested' | 'notApplicable';
  detail: string;
}
export const checkNetwork = () => desktopInvoke<DiagnosticCheck[]>('check_network');
export const is_proxy_running = () => desktopInvoke<boolean>('is_proxy_running');
export const getProviders = () => desktopInvoke<{ providers: Record<string, Provider> }>('get_providers');
export const getProviderProxies = (providerName: string) => desktopInvoke<Provider>('get_provider_proxies', { providerName });
export const triggerProviderHealthCheck = (providerName: string) => desktopInvoke<void>('trigger_provider_health_check', { providerName });
export const getTunStatus = () => desktopInvoke<boolean>('get_tun_status');
export const toggleTun = (enabled: boolean) => desktopInvoke<void>('toggle_tun', { enabled });
export const getUptime = () => desktopInvoke<number>('get_uptime');
export const closeAllConnections = () => desktopInvoke<void>('close_all_connections');
export const getControllerConfig = () => desktopInvoke<ControllerConfig>('get_controller_config');

export function webSocketUrl(config: ControllerConfig, endpoint: string): string {
  const url = new URL(endpoint, config.wsUrl);
  if (config.secret) url.searchParams.set('token', config.secret);
  return url.toString();
}
