use std::{
    error::Error as StdError,
    fmt::{Display, Formatter},
    sync::Arc,
};
use tracing::debug;

#[derive(Debug, Clone)]
pub struct Error {
    inner: Arc<anyhow::Error>,
}

impl<E> From<E> for Error
where
    E: StdError + Send + Sync + 'static,
{
    #[cold]
    fn from(error: E) -> Self {
        debug!("`{error}`");
        Self {
            inner: Arc::new(error.into()),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

#[macro_export]
macro_rules! anyio {
    ($($tt:tt)*) => {
        $crate::utils::Error::from(std::io::Error::other(format!($($tt)*)))
    };
}

pub use anyio;

// impl StdError for Error {
//     fn source(&self) -> Option<&(dyn StdError + 'static)> {
//         self.inner.source()
//     }
// }

pub type Result<T, E = Error> = std::result::Result<T, E>;
