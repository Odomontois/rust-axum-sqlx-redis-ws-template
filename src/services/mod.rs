pub mod cars;
pub mod parts;

#[cfg(test)]
use std::{future::Future, pin::Pin, result::Result};
#[cfg(test)]
pub(crate) fn mock_result<A: Send + 'static, E: Send + 'static>(
    result: A,
) -> Pin<Box<dyn Future<Output = Result<A, E>> + Send>> {
    use std::future::ready;

    Box::pin(ready(Ok(result)))
}
