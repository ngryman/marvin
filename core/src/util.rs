/// Safe
pub trait Safe: Send + Sync + 'static {}
impl<T: Send + Sync + 'static> Safe for T {}
