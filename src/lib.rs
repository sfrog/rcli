mod command;
mod process;
mod utils;

pub use command::{
    Base64SubCommand, CsvOpts, DecodeOpts, EncodeOpts, GenPassOpts, GenerateOpts, HttpServeOpts,
    HttpSubCommand, Opts, SubCommand, TextSignFormat, TextSignOpts, TextSubCommand, TextVerifyOpts,
};
use enum_dispatch::enum_dispatch;
pub use process::{
    process_base64_decode, process_base64_encode, process_csv, process_gen_pass,
    process_http_serve, process_text_generate, process_text_sign, process_text_verify,
};
pub use utils::*;

#[allow(async_fn_in_trait)]
#[enum_dispatch]
pub trait CmdExecutor {
    async fn execute(self) -> anyhow::Result<()>;
}
