//! 代理进程管理模块
//! 
//! 该模块负责代理进程的启动和停止操作，包括：
//! 1. 启动 Mihomo 代理进程
//! 2. 停止 Mihomo 代理进程
//! 3. 管理代理进程的状态

use std::sync::Mutex;
use std::process::Child;

/// 应用状态结构体
/// 
/// 用于管理应用的全局状态，主要是代理进程的状态
pub struct AppState {
    /// 代理进程的互斥锁，用于线程安全地访问和修改进程状态
    pub proxy_process: Mutex<Option<Child>>,
}
