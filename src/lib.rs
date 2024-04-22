mod command;
mod process;

pub use command::{Base64SubCommand, Opts, SubCommand};
pub use process::{process_base64_decode, process_base64_encode, process_csv, process_gen_pass};
