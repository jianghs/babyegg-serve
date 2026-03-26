//! 通用 HTTP 中间件模块。
//!
//! 这里放置可被多个业务服务复用的中间件实现，
//! 当前包含 request id 注入能力。

/// request id 中间件。
pub mod request_id;
