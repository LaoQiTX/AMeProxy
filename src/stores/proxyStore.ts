import { defineStore } from 'pinia';
import type { Connection, Rule, Log, ProxyGroup, Subscription, Proxy, TrafficData } from '../types';

// 条件导入invoke函数
let invoke: any;
try {
  // 动态导入Tauri API
  import('@tauri-apps/api/core').then(({ invoke: tauriInvoke }) => {
    invoke = tauriInvoke;
  });
} catch (error) {
  console.warn('Tauri环境未初始化，使用模拟invoke函数');
}

// 确保invoke始终有值
if (!invoke) {
  invoke = async (command: string, args: any) => {
    console.log(`模拟调用Tauri命令: ${command}`, args);
    return Promise.resolve('模拟成功');
  };
}
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
    // 切换连接状态
    async toggleConnection() {
      try {
        if (!this.isConnected) {
          await invoke('start_proxy');
          this.isConnected = true;
          this.startPolling();
          this.connectWebSocket();
          this.connectLogWebSocket();
          this.connectConnectionsWebSocket();
          this.fetchRules();
          this.fetchProviders(); // 获取订阅信息
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
        const res = await fetch('http://127.0.0.1:9090/providers/proxies');
        console.log('订阅信息请求状态:', res.status);
        if (res.ok) {
          const data = await res.json();
          console.log('获取到的订阅信息:', data);
          
          // 清空现有订阅
          this.subscriptions = [];
          
          // 处理每个订阅
          if (data.providers) {
            for (const name in data.providers) {
              const provider = data.providers[name];
              console.log('处理订阅:', name, provider);
              const subscription: Subscription = {
                name: name,
                url: provider.proxyProvider?.url || provider.url || '',
                count: provider.proxies?.length || 0,
                updateTime: new Date().toLocaleString().slice(0, 16)
              };
              this.subscriptions.push(subscription);
            }
          } else {
            console.log('没有获取到订阅信息');
          }
          console.log('最终订阅列表:', this.subscriptions);
        } else {
          console.error('获取订阅信息失败:', res.statusText);
        }
      } catch (err) {
        console.error("Failed to fetch providers:", err);
      }
    },
    async fetchRules() {
      try {
        const res = await fetch('http://127.0.0.1:9090/rules');
        if (res.ok) {
          const data = await res.json();
          this.rules = (data.rules || []).map((r: any) => ({
            type: r.type,
            payload: r.payload,
            strategy: r.proxy
          }));
        }
      } catch (err) {}
    },
    connectConnectionsWebSocket() {
      if (this.connWs) return;
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
        } catch (e) {}
      };
      this.connWs.onclose = () => {
        this.connWs = null;
        if (this.isConnected) {
          setTimeout(() => this.connectConnectionsWebSocket(), 3000);
        }
      };
    },
    disconnectConnectionsWebSocket() {
      if (this.connWs) {
        this.connWs.close();
        this.connWs = null;
      }
    },
    connectLogWebSocket() {
      if (this.logWs) return;
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
        } catch (e) {}
      };
      this.logWs.onclose = () => {
        this.logWs = null;
        if (this.isConnected) {
          setTimeout(() => this.connectLogWebSocket(), 3000);
        }
      };
    },
    disconnectLogWebSocket() {
      if (this.logWs) {
        this.logWs.close();
        this.logWs = null;
      }
    },
    connectWebSocket() {
      if (this.ws) return;
      this.ws = new WebSocket('ws://127.0.0.1:9090/traffic');
      this.ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          this.trafficData = {
            up: this.formatBytes(data.up) + '/s',
            down: this.formatBytes(data.down) + '/s'
          };
        } catch (e) {}
      };
      this.ws.onclose = () => {
        this.ws = null;
        if (this.isConnected) {
          setTimeout(() => this.connectWebSocket(), 3000);
        }
      };
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
        const res = await fetch('http://127.0.0.1:9090/proxies');
        if (res.ok) {
          const data = await res.json();
          // 更新代理组和节点
          const groups: ProxyGroup[] = [];
          const nodes: Proxy[] = [];
          
          for (const key in data.proxies) {
            const p = data.proxies[key];
            if (p.all && p.all.length > 0) {
              groups.push({
                name: p.name,
                type: p.type,
                options: p.all,
                selected: p.now || ''
              });
            } else if (p.type !== 'Selector' && p.type !== 'URLTest' && p.type !== 'Fallback' && p.type !== 'LoadBalance' && p.type !== 'Direct' && p.type !== 'Reject') {
              nodes.push({
                name: p.name,
                type: p.type,
                delay: p.history && p.history.length > 0 ? p.history[p.history.length - 1].delay : 0,
                region: p.name.substring(0, 2)
              });
            }
          }
          this.proxyGroups = groups;
          this.proxies = nodes;
        }
      } catch (err) {
        // console.error("Fetch proxies error:", err);
      }
    },
    async switchProxy(groupName: string, proxyName: string) {
      try {
        const res = await fetch(`http://127.0.0.1:9090/proxies/${encodeURIComponent(groupName)}`, {
          method: 'PUT',
          headers: {
            'Content-Type': 'application/json'
          },
          body: JSON.stringify({ name: proxyName })
        });
        if (res.ok) {
          // fetchProxies(); // will be updated by polling
          const group = this.proxyGroups.find(g => g.name === groupName);
          if (group) {
            group.selected = proxyName;
          }
        }
      } catch (err) {
        console.error("Failed to switch proxy:", err);
      }
    },
    // 模拟流量数据 - 暂时保留
    simulateTraffic() {
      if (!this.isConnected) return;
      this.trafficData = {
        up: (Math.random() * 100).toFixed(1) + ' KB/s',
        down: (Math.random() * 500).toFixed(1) + ' KB/s'
      };
      setTimeout(() => this.simulateTraffic(), 2000);
    },
    // 测试延迟
    async testLatency() {
      this.isTesting = true;
      try {
        const promises = this.proxies.map(async (p) => {
          try {
            const res = await fetch(`http://127.0.0.1:9090/proxies/${encodeURIComponent(p.name)}/delay?timeout=5000&url=http%3A%2F%2Fwww.gstatic.com%2Fgenerate_204`);
            if (res.ok) {
              const data = await res.json();
              p.delay = data.delay;
            } else {
              p.delay = -1;
            }
          } catch {
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
        
        // 首先添加订阅到配置文件
        console.log('调用add_proxy_provider...');
        await invoke('add_proxy_provider', { name, url });
        console.log('add_proxy_provider调用成功');
        
        // 如果代理已连接，重启代理以加载新配置
        if (this.isConnected) {
          console.log('代理已连接，重启代理...');
          await invoke('stop_proxy');
          console.log('代理已停止');
          await invoke('start_proxy');
          console.log('代理已启动');
          // 等待代理启动后获取订阅信息
          setTimeout(() => {
            console.log('等待2秒后获取订阅信息...');
            this.fetchProviders();
            this.fetchProxies();
          }, 2000);
        }
        
        // 直接从API获取最新的订阅信息
        console.log('直接获取订阅信息...');
        await this.fetchProviders();
        console.log('获取订阅信息完成');
        
        console.log('获取节点信息...');
        await this.fetchProxies();
        console.log('获取节点信息完成');
        
        console.log('当前订阅列表:', this.subscriptions);
        console.log('当前节点列表:', this.proxies);
        
        alert('订阅已导入并加载成功');
      } catch (err) {
        console.error("Failed to import subscription:", err);
        alert("订阅导入失败: " + err);
      }
    },
    
    // 删除订阅
    async removeSubscription(name: string) {
      try {
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
