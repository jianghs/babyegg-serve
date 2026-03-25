/// 基础状态 trait 的占位。
/// 真正的业务状态通常在各自服务里定义，这里先不强行抽象太深。
pub trait AppStateMarker: Clone + Send + Sync + 'static {}

impl<T> AppStateMarker for T where T: Clone + Send + Sync + 'static {}
