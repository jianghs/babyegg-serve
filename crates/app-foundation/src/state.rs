/// 基础状态 trait 的占位。
/// 真正的业务状态通常在各自服务里定义，这里先不强行抽象太深。
pub trait AppStateMarker: Clone + Send + Sync + 'static {}

/// 为满足基础状态约束的类型自动实现标记 trait。
impl<T> AppStateMarker for T where T: Clone + Send + Sync + 'static {}
