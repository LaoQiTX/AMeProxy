import { defineStore } from 'pinia';
import type { Connection, Rule, Log, ProxyGroup, Subscription, Proxy, TrafficData } from '../types';

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

export const useProxyStore = defineStore('proxy', {
  state: () => ({
    // 当前标签页
    currentTab: 'dashboard' as string,
    // 连接状态
    isConnected: false,
    // 选中的内核
    selectedKernel: 'Mihomo',
    // 内核列表
    kernels: ['Mihomo'],
    // 测试状态
    isTesting: false,
    // TUN 模式
    tunMode: false,
    // 连接列表
    connections: [] as Connection[],
    // 规则列表
    rules: [] as Rule[],
    // 日志列表
    logs: [] as Log[],
    // 代理组
    proxyGroups: [] as ProxyGroup[],
    // 订阅列表
    subscriptions: [] as Subscription[],
    // 代理节点
    proxies: [] as Proxy[],
    // 流量数据
    trafficData: {
      up: '0 KB/s',
      down: '0 KB/s'
    } as TrafficData,
    // API Polling interval
    pollInterval: null as any,
    ws: null as WebSocket | null,
    logWs: null as WebSocket | null,
    connWs: null as WebSocket | null
  }),
  actions: {
    // 初始化应用
    async initialize() {
      try {
        console.log('Starting initialize...');
        // 暂时注释掉初始化逻辑，避免错误
        // const invoke = await getInvoke();
        // console.log('Got invoke function:', typeof invoke);
        // 
        // // 检查代理是否已经在运行
        // if (typeof invoke === 'function') {
        //   console.log('Calling is_proxy_running...');
        //   const isRunning = await invoke('is_proxy_running');
        //   console.log('is_proxy_running result:', isRunning);
        //   
        //   if (isRunning) {
        //     // 不自动连接到代理，只获取基本信息
        //     // this.isConnected = true;
        //     // this.startPolling();
        //     // this.connectWebSocket();
        //     // this.connectLogWebSocket();
        //     // this.connectConnectionsWebSocket();
        //     // this.fetchRules();
        //     // this.fetchProviders(); // 获取订阅信息
        //     // this.fetchProxies(); // 获取代理节点
        //   }
        // } else {
        //   console.error('Invoke function is not a function:', invoke);
        // }
      } catch (err) {
        console.error("Failed to initialize:", err);
      }
    },

    // 切换连接状态
    async toggleConnection() {
      try {
        const invoke = await getInvoke();
        if (!this.isConnected) {
          await invoke('start_proxy');
          this.isConnected = true;
          this.startPolling();
          this.connectWebSocket();
          this.connectLogWebSocket();
          this.connectConnectionsWebSocket();
          this.fetchRules();
          this.fetchProviders(); // 获取订阅信息
          this.fetchProxies(); // 获取代理节点
        } else {
          await invoke('stop_proxy');
          this.isConnected = false;
          this.stopPolling();
          this.disconnectWebSocket();
          this.disconnectLogWebSocket();
          this.disconnectConnectionsWebSocket();
          this.trafficData = { up: '0 KB/s', down: '0 KB/s' };
          this.connections = [];
          this.rules = [];
        }
      } catch (err) {
        console.error("Failed to toggle connection:", err);
        alert("操作失败: " + err);
      }
    },
    
    // 获取订阅信息
    async fetchProviders() {
      try {
        console.log('开始获取订阅信息...');
        
        // 先尝试从 config.yaml 文件中读取订阅信息
        try {
          const invoke = await getInvoke();
          const configData = await invoke('get_config');
          console.log('从 config.yaml 获取的订阅信息:', configData);
          
          // 清空现有订阅
          this.subscriptions = [];
          
          // 处理 config.yaml 中的订阅
          if (configData['proxy-providers'] && typeof configData['proxy-providers'] === 'object') {
            for (const name in configData['proxy-providers']) {
              const provider = configData['proxy-providers'][name];
              console.log('处理 config.yaml 中的订阅:', name, provider);
              
              // 跳过 'override'，它不是订阅，而是订阅的属性
              if (name === 'override') {
                continue;
              }
              
              // 只有当 provider 有 url 属性时，才认为它是一个订阅
              if (provider.url) {
                const subscription: Subscription = {
                  name: name,
                  url: provider.url || '',
                  count: 0, // 从 config.yaml 中无法获取节点数量，需要从 API 获取
                  updateTime: new Date().toLocaleString().slice(0, 16)
                };
                this.subscriptions.push(subscription);
              }
            }
          }
          
          console.log('从 config.yaml 获取的订阅列表:', this.subscriptions);
        } catch (configErr) {
          console.error("Failed to get config from file:", configErr);
        }
        
        // 然后尝试从 API 获取订阅信息，更新节点数量
        try {
          const invoke = await getInvoke();
          const data = await invoke('get_providers');
          console.log('从 API 获取到的订阅信息:', data);
          
          // 处理每个订阅，更新节点数量
          if ((data as any).providers) {
            const providersData = (data as any).providers;
            for (const name in providersData) {
              // 跳过 'override'，它不是订阅，而是订阅的属性
              if (name === 'override') {
                continue;
              }
              
              const provider = providersData[name];
              console.log('处理 API 订阅:', name, provider);
              
              // 查找现有的订阅
              const existingSubscription = this.subscriptions.find(s => s.name === name);
              if (existingSubscription) {
                // 更新节点数量
                existingSubscription.count = provider.proxies?.length || 0;
              } else if (provider.vehicleType === 'HTTP') {
                // 添加新的订阅
                const subscription: Subscription = {
                  name: name,
                  url: provider.proxyProvider?.url || provider.url || '',
                  count: provider.proxies?.length || 0,
                  updateTime: new Date().toLocaleString().slice(0, 16)
                };
                this.subscriptions.push(subscription);
              }
            }
          } else if (data && typeof data === 'object') {
            // 直接处理数据，可能是 API 响应格式变化
            for (const name in data) {
              // 跳过 'override'，它不是订阅，而是订阅的属性
              if (name === 'override') {
                continue;
              }
              
              const provider = (data as any)[name];
              console.log('处理 API 订阅（直接）:', name, provider);
              
              // 查找现有的订阅
              const existingSubscription = this.subscriptions.find(s => s.name === name);
              if (existingSubscription) {
                // 更新节点数量
                existingSubscription.count = provider.proxies?.length || 0;
              } else if (provider.vehicleType === 'HTTP') {
                // 添加新的订阅
                const subscription: Subscription = {
                  name: name,
                  url: provider.proxyProvider?.url || provider.url || '',
                  count: provider.proxies?.length || 0,
                  updateTime: new Date().toLocaleString().slice(0, 16)
                };
                this.subscriptions.push(subscription);
              }
            }
          }
        } catch (apiErr) {
          console.error("Failed to fetch providers from API:", apiErr);
          // API 调用失败，保留从 config.yaml 获取的订阅信息
        }
        
        console.log('最终订阅列表:', this.subscriptions);
      } catch (err) {
        console.error("Failed to fetch providers:", err);
        // 保留空订阅列表
        this.subscriptions = [];
        console.log('最终订阅列表（空）:', this.subscriptions);
      }
    },
    async fetchRules() {
      try {
        const invoke = await getInvoke();
        const data = await invoke('get_rules');
        console.log('Fetch rules data:', data);
        this.rules = ((data as any).rules || []).map((r: any) => ({
          type: r.type || 'Unknown',
          payload: r.payload || '',
          strategy: r.proxy || ''
        }));
        console.log('Updated rules:', this.rules);
      } catch (err) {
        console.error("Failed to fetch rules:", err);
        // 错误时清空数据
        this.rules = [];
      }
    },
    connectConnectionsWebSocket() {
      if (this.connWs) return;
      try {
        this.connWs = new WebSocket('ws://127.0.0.1:9090/connections');
        this.connWs.onmessage = (event) => {
          try {
            const data = JSON.parse(event.data);
            this.connections = (data.connections || []).map((c: any) => ({
              id: c.id,
              host: c.metadata.host || c.metadata.destinationIP,
              ip: c.metadata.destinationIP,
              process: c.metadata.processPath ? c.metadata.processPath.split('\\').pop().split('/').pop() : '',
              rule: c.rule,
              group: c.chains ? c.chains[0] : '',
              speed: this.formatBytes(c.downloadSpeed || 0) + '/s',
              time: new Date(c.start).toLocaleTimeString()
            }));
          } catch (e) {
            console.error("WebSocket message error:", e);
          }
        };
        this.connWs.onclose = () => {
          this.connWs = null;
          if (this.isConnected) {
            setTimeout(() => this.connectConnectionsWebSocket(), 3000);
          }
        };
        this.connWs.onerror = (error) => {
          console.warn("Connections WebSocket error:", error);
          this.connWs = null;
        };
      } catch (error) {
        console.warn("Failed to connect to connections WebSocket:", error);
        this.connWs = null;
      }
    },
    disconnectConnectionsWebSocket() {
      if (this.connWs) {
        this.connWs.close();
        this.connWs = null;
      }
    },
    connectLogWebSocket() {
      if (this.logWs) return;
      try {
        this.logWs = new WebSocket('ws://127.0.0.1:9090/logs?level=info');
        this.logWs.onmessage = (event) => {
          try {
            const data = JSON.parse(event.data);
            this.logs.unshift({
              time: new Date().toLocaleTimeString(),
              level: data.type,
              msg: data.payload
            });
            if (this.logs.length > 200) {
              this.logs.pop();
            }
          } catch (e) {
            console.error("WebSocket message error:", e);
          }
        };
        this.logWs.onclose = () => {
          this.logWs = null;
          if (this.isConnected) {
            setTimeout(() => this.connectLogWebSocket(), 3000);
          }
        };
        this.logWs.onerror = (error) => {
          console.warn("Log WebSocket error:", error);
          this.logWs = null;
        };
      } catch (error) {
        console.warn("Failed to connect to log WebSocket:", error);
        this.logWs = null;
      }
    },
    disconnectLogWebSocket() {
      if (this.logWs) {
        this.logWs.close();
        this.logWs = null;
      }
    },
    connectWebSocket() {
      if (this.ws) return;
      try {
        this.ws = new WebSocket('ws://127.0.0.1:9090/traffic');
        this.ws.onmessage = (event) => {
          try {
            const data = JSON.parse(event.data);
            this.trafficData = {
              up: this.formatBytes(data.up) + '/s',
              down: this.formatBytes(data.down) + '/s'
            };
          } catch (e) {
            console.error("WebSocket message error:", e);
          }
        };
        this.ws.onclose = () => {
          this.ws = null;
          if (this.isConnected) {
            setTimeout(() => this.connectWebSocket(), 3000);
          }
        };
        this.ws.onerror = (error) => {
          console.warn("Traffic WebSocket error:", error);
          this.ws = null;
        };
      } catch (error) {
        console.warn("Failed to connect to traffic WebSocket:", error);
        this.ws = null;
      }
    },
    disconnectWebSocket() {
      if (this.ws) {
        this.ws.close();
        this.ws = null;
      }
    },
    formatBytes(bytes: number) {
      if (bytes === 0) return '0 B';
      const k = 1024;
      const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
      const i = Math.floor(Math.log(bytes) / Math.log(k));
      return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    },
    startPolling() {
      if (this.pollInterval) return;
      this.pollInterval = setInterval(() => {
        this.fetchProxies();
      }, 2000);
    },
    stopPolling() {
      if (this.pollInterval) {
        clearInterval(this.pollInterval);
        this.pollInterval = null;
      }
    },
    async fetchProxies() {
      try {
        const invoke = await getInvoke();
        const data = await invoke('get_proxies');
        console.log('Fetch proxies data:', data);
        // 更新代理组和节点
        const groups: ProxyGroup[] = [];
        const nodes: Proxy[] = [];
        
        // 检查数据格式
        if (data && typeof data === 'object') {
          const proxiesData = (data as any).proxies;
          if (proxiesData && typeof proxiesData === 'object') {
            for (const key in proxiesData) {
              const p = proxiesData[key];
              if (p && typeof p === 'object') {
                if (p.all && Array.isArray(p.all) && p.all.length > 0) {
                  groups.push({
                    name: p.name || key,
                    type: p.type || 'Unknown',
                    options: p.all,
                    selected: p.now || ''
                  });
                } else if (p.type && p.type !== 'Selector' && p.type !== 'URLTest' && p.type !== 'Fallback' && p.type !== 'LoadBalance' && p.type !== 'Direct' && p.type !== 'Reject') {
                  nodes.push({
                    name: p.name || key,
                    type: p.type,
                    delay: p.history && Array.isArray(p.history) && p.history.length > 0 ? p.history[p.history.length - 1].delay : 0,
                    region: p.name ? p.name.substring(0, 2) : ''
                  });
                }
              }
            }
          }
        }
        
        this.proxyGroups = groups;
        this.proxies = nodes;
        console.log('Updated proxy groups:', groups);
        console.log('Updated proxies:', nodes);
      } catch (err) {
        console.error("Fetch proxies error:", err);
        // 错误时清空数据，避免显示旧数据
        this.proxyGroups = [];
        this.proxies = [];
      }
    },
    async switchProxy(groupName: string, proxyName: string) {
      try {
        const invoke = await getInvoke();
        await invoke('change_proxy', { group: groupName, proxy: proxyName });
        // fetchProxies(); // will be updated by polling
        const group = this.proxyGroups.find(g => g.name === groupName);
        if (group) {
          group.selected = proxyName;
        }
      } catch (err) {
        console.error("Failed to switch proxy:", err);
      }
    },

    // 测试延迟
    async testLatency() {
      this.isTesting = true;
      try {
        const invoke = await getInvoke();
        const promises = this.proxies.map(async (p) => {
          try {
            const delay = await invoke('test_proxy', { proxy: p.name });
            p.delay = delay;
          } catch (err) {
            console.error("Test proxy error:", err);
            p.delay = -1;
          }
        });
        await Promise.all(promises);
      } catch (err) {
        console.error("Test latency error:", err);
      } finally {
        this.isTesting = false;
      }
    },
    // 添加订阅
    async addSubscription(name: string, url: string) {
      if (!url || !name) return;
      try {
        const invoke = await getInvoke();
        await invoke('add_proxy_provider', { name, url });
        
        const existing = this.subscriptions.find(s => s.name === name);
        if (existing) {
          existing.updateTime = new Date().toLocaleString().slice(0, 16);
        } else {
          this.subscriptions.push({
            name,
            url,
            count: 0,
            updateTime: new Date().toLocaleString().slice(0, 16)
          });
        }
        if (this.isConnected) {
          await invoke('stop_proxy');
          await invoke('start_proxy');
        }
        alert('订阅已添加并写入 config.yaml');
      } catch (err) {
        console.error("Failed to add subscription:", err);
        alert("订阅添加失败: " + err);
      }
    },
    
    // 更新订阅
    async updateSubscription(oldName: string, newName: string, url: string) {
      if (!url || !newName) return;
      try {
        const invoke = await getInvoke();
        await invoke('update_proxy_provider', { oldName, newName, url });
        
        const existing = this.subscriptions.find(s => s.name === oldName);
        if (existing) {
          existing.name = newName;
          existing.url = url;
          existing.updateTime = new Date().toLocaleString().slice(0, 16);
        }
        if (this.isConnected) {
          await invoke('stop_proxy');
          await invoke('start_proxy');
        }
        alert('订阅已更新并写入 config.yaml');
      } catch (err) {
        console.error("Failed to update subscription:", err);
        alert("订阅更新失败: " + err);
      }
    },
    
    // 导入订阅
    async importSubscription(name: string, url: string) {
      if (!url || !name) return;
      try {
        console.log('开始导入订阅:', name, url);
        const invoke = await getInvoke();
        
        // 首先添加订阅到配置文件
        console.log('调用add_proxy_provider...');
        await invoke('add_proxy_provider', { name, url });
        console.log('add_proxy_provider调用成功');
        
        // 确保代理已启动
        if (!this.isConnected) {
          console.log('代理未连接，启动代理...');
          await invoke('start_proxy');
          this.isConnected = true;
          this.startPolling();
          this.connectWebSocket();
          this.connectLogWebSocket();
          this.connectConnectionsWebSocket();
          console.log('代理已启动');
        } else {
          console.log('代理已连接，重启代理...');
          await invoke('stop_proxy');
          console.log('代理已停止');
          await invoke('start_proxy');
          console.log('代理已启动');
        }
        
        // 获取订阅信息
        console.log('获取订阅信息...');
        await this.fetchProviders();
        console.log('获取订阅信息完成');
        
        // 获取节点信息
        console.log('获取节点信息...');
        await this.fetchProxies();
        console.log('获取节点信息完成');
        
        console.log('当前订阅列表:', this.subscriptions);
        console.log('当前节点列表:', this.proxies);
        
        // 检查订阅是否真的添加成功
        const subscriptionExists = this.subscriptions.some(s => s.name === name);
        if (subscriptionExists) {
          alert('订阅已导入并加载成功');
        } else {
          alert('订阅已添加到配置文件，但可能需要一些时间来下载和解析。请稍后刷新页面查看。');
        }
      } catch (err) {
        console.error("Failed to import subscription:", err);
        alert("订阅导入失败: " + err);
        throw err;
      }
    },
    
    // 删除订阅
    async removeSubscription(name: string) {
      try {
        const invoke = await getInvoke();
        await invoke('remove_proxy_provider', { name });
        this.subscriptions = this.subscriptions.filter(s => s.name !== name);
        if (this.isConnected) {
          await invoke('stop_proxy');
          await invoke('start_proxy');
        }
        alert('订阅已删除并更新 config.yaml');
      } catch (err) {
        console.error("Failed to remove subscription:", err);
        alert("订阅删除失败: " + err);
      }
    },
    // 切换 TUN 模式
    toggleTunMode() {
      this.tunMode = !this.tunMode;
    },
    // 切换内核
    setKernel(kernel: string) {
      this.selectedKernel = kernel;
    },
    // 切换标签页
    setCurrentTab(tab: string) {
      this.currentTab = tab;
    }
  }
});
