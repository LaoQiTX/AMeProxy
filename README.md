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

## 📋 待办事项

### 核心功能
- [ ] **内核管理**
  - [ ] 关闭应用时自动关闭内核进程
  - [ ] 自动下载并更新 mihomo 内核
  - [ ] 后台运行支持

### 代理管理
- [ ] **连接监控**
  - [ ] 实时连接状态展示
  - [ ] 连接详情查看
  - [ ] 连接统计分析

- [ ] **测速功能**
  - [ ] 单个节点测速
  - [ ] 批量节点测速
  - [ ] 测速结果排序

- [ ] **TUN 模式**
  - [ ] TUN 模式切换开关
  - [ ] TUN 模式配置优化

### 配置与日志
- [ ] **配置管理**
  - [ ] 自定义配置文件编辑
  - [ ] 配置文件导入/导出
  - [ ] 配置模板管理

- [ ] **日志系统**
  - [ ] 实时运行日志查看
  - [ ] 日志级别筛选
  - [ ] 日志导出功能

### 界面与用户体验
- [ ] **运行信息**
  - [ ] 运行时长显示
  - [ ] 系统资源占用监控
  - [ ] 网络流量统计

- [ ] **主题与样式**
  - [ ] 更多主题配色方案
  - [ ] 自定义主题支持
  - [ ] 响应式布局优化
  
## 缺陷待修复
### 功能缺陷：
- [ ] 关闭应用时内核不会自动关闭
- [ ] 缺少内核自动下载功能
- [ ] 缺少测速、实时连接监控、自定义配置文件、运行时长统计、后台运行等功能
### 技术缺陷：
- [ ] 前端同时使用 Vue 3 和 React，技术栈混乱
- [ ] 代码组织不够清晰
- [ ] 依赖管理存在问题
- [ ] 后端功能实现不完整
### 用户体验问题：
- [ ] 界面功能不完整
- [ ] 缺少用户引导和帮助信息
- [ ] 响应式设计可能存在问题
- [ ] 缺少完善的错误处理机制

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
- \[bugdey]<https://gh.bugdey.us.kg> - github下载加速。

***

**AMeProxy** - 更清新的科学上网。(*^\_^*)
