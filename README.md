<h1 align="center">
  <img src="./icon.png" alt="AMeProxy" width="128" />
  <br>
   AMeProxy - 雨代理 🌧️
  <br>
  基于 <a href="https://github.com/MetaCubeX/mihomo">mihomo内核</a>的清新代理工具
</h1>

## 项目目前仍在开发状态
## AMeProxy  其中的AMe取自日语中的雨（あめ）的五十音。

## 🌟 技术栈

- **前端框架**: [Vue 3 (Composition API)](https://vuejs.org/)
- **样式处理**: [Tailwind CSS 4.0](https://tailwindcss.com/)
- **图标库**: [Lucide Vue Next](https://lucide.dev/)
- **动画库**: [Motion (Native CSS Transitions)](https://motion.dev/)
- **构建工具**: [Vite 6](https://vitejs.dev/)
- **目标平台**: Web / [Tauri](https://tauri.app/)
- **代理内核**: [mihomo](https://github.com/MetaCubeX/mihomo) 


## 🚀 核心功能实现


1. **多维仪表盘**: 实时流量监控（上传/下载）、系统资源占用、连接延迟展示。
2. **TUN 模式支持**: 在设置中提供全局流量接管开关。
3. **个性化主题**: 内置四种清新配色方案（薄荷绿、樱花粉、薰衣草、晴空蓝），支持实时切换。
4. **终端日志**: 实时输出内核运行日志，支持日志级别着色。
5. ……

## ToDo
- [ ] 查看日志
- [ ] 自定义配置文件
- [ ] 关闭应用时内核居然不会自动关闭 神了
- [ ] 运行时长
- [ ] 测速
- [ ] 实时连接
- [ ] 运行日志
- [ ] 切换tun模式 
- [ ] 自动下载内核
- [ ] 后台运行



## 🛠️ 部署与运行

### 1. 环境准备
确保您的开发环境已安装 [Node.js](https://nodejs.org/) (建议 v18+)。

### 2. 安装依赖
```bash
npm install
```

### 3. 本地开发
```bash
npm run dev
```
应用将运行在 `http://localhost:3000`。

### 4. 生产构建
```bash
npm run build
```
构建产物将输出在 `dist/` 目录中。

## 📦 Tauri 打包建议

若要将本项目打包为桌面应用，请遵循以下步骤：

1. 安装 Tauri CLI: `npm install -D @tauri-apps/cli`
2. 初始化 Tauri: `npx tauri init`
3. 配置 `tauri.conf.json`:
   - `beforeDevCommand`: `npm run dev`
   - `beforeBuildCommand`: `npm run build`
   - `distDir`: `../dist`
4. 运行开发版: `npx tauri dev`
5. 构建安装包: `npx tauri build`

> 鸣谢
- [mihomo](https://github.com/MetaCubeX/mihomo) - 代理内核。
- [bugdey]https://gh.bugdey.us.kg - github下载加速。

---

**AMeProxy** - 更清新的科学上网。(*^_^*)
