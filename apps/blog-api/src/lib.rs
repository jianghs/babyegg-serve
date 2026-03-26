//! `blog-api` 是示例博客服务的业务主 crate。
//!
//! 它负责组装应用配置、路由、领域模块与数据库访问层，
//! 并对外暴露可供 `main.rs` 与集成测试复用的应用构建入口。

/// 应用路由与服务构建入口。
pub mod app;
/// 服务级配置模型。
pub mod config;
/// 数据访问层与仓储模块。
pub mod db;
/// 按领域划分的业务模块集合。
pub mod modules;
/// 应用共享状态定义。
pub mod state;
