mod command;
mod process;
mod utils;

use std::path::Path;

use enum_dispatch::enum_dispatch;

pub use command::*;
pub use process::*;
pub use utils::*;

#[allow(async_fn_in_trait)]
#[enum_dispatch]
pub trait CmdExecutor {
    async fn execute(self) -> anyhow::Result<()>;
}

pub trait KeyLoader {
    /// Load the key from the given path
    fn load(path: impl AsRef<Path>) -> anyhow::Result<Self>
    where
        Self: Sized;
}
