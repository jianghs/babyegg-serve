//! `app-foundation` 提供多个业务服务可复用的基础设施能力。
//!
//! 这里集中放置配置加载、错误模型、国际化、HTTP 响应包装、
//! 通用查询参数与中间件等横切能力，供各应用 crate 直接复用。

/// 基础配置模型与环境变量加载逻辑。
pub mod config;
/// 通用应用错误与 HTTP 错误响应转换。
pub mod error;
/// 机器可读错误码定义。
pub mod error_code;
/// 国际化消息与翻译入口。
pub mod i18n;
/// 支持的语言区域定义。
pub mod locale;
/// 日志初始化相关能力。
pub mod logging;
/// 通用 HTTP 中间件集合。
pub mod middleware;
/// 通用列表查询参数与分页归一化工具。
pub mod query;
/// 通用成功响应与分页响应结构。
pub mod response;
/// 应用共享状态抽象。
pub mod state;
/// 参数校验结果与校验原因模型。
pub mod validation;
/// 通用 Web 接口模块。
pub mod web;

/// 基础配置的对外重导出。
pub use config::BaseConfig;
/// 通用应用错误的对外重导出。
pub use error::AppError;
/// 机器可读错误码的对外重导出。
pub use error_code::ErrorCode;
/// 语言区域枚举的对外重导出。
pub use locale::Locale;
/// 通用列表查询工具的对外重导出。
pub use query::{ListQuery, NormalizedListQuery, SortOrder};
/// 通用响应结构的对外重导出。
pub use response::{ApiResponse, PageResponse};
/// 参数校验结果模型的对外重导出。
pub use validation::{ValidationDetail, ValidationReason};
