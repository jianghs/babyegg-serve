//! 授权与 RBAC 模块。

/// 角色与权限校验工具。
pub mod authorization;
/// 角色权限目录常量。
pub mod catalog;
/// 访问上下文模型。
pub mod context;
/// 角色与权限键常量。
pub mod keys;
/// RBAC 业务服务。
pub mod service;
