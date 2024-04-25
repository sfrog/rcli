mod base64;
mod csv;
mod gen_pass;
mod http;
mod text;

use std::path::{Path, PathBuf};

use clap::Parser;

use crate::CmdExecutor;

use self::{csv::CsvOpts, gen_pass::GenPassOpts};

pub use self::{
    base64::Base64Format, base64::Base64SubCommand, csv::OutputFormat, http::HttpSubCommand,
    text::TextSignFormat, text::TextSubCommand,
};

#[derive(Parser, Debug)]
#[command(name = "rcli", version, author, about, long_about = None)]
pub struct Opts {
    #[command(subcommand)]
    pub cmd: SubCommand,
}

#[derive(Parser, Debug)]
pub enum SubCommand {
    #[command(name = "csv", about = "Convert CSV to JSON")]
    Csv(CsvOpts),
    #[command(name = "genpass", about = "Generate a random password")]
    GenPass(GenPassOpts),
    #[command(subcommand, about = "Base64 encode/decode")]
    Base64(Base64SubCommand),
    #[command(subcommand, about = "Text signing and verification")]
    Text(TextSubCommand),
    #[command(subcommand, about = "HTTP server")]
    Http(HttpSubCommand),
}

impl CmdExecutor for SubCommand {
    async fn execute(self) -> anyhow::Result<()> {
        match self {
            SubCommand::Csv(opts) => opts.execute().await?,
            SubCommand::GenPass(opts) => opts.execute().await?,
            SubCommand::Base64(sub_cmd) => sub_cmd.execute().await?,
            SubCommand::Text(sub_cmd) => sub_cmd.execute().await?,
            SubCommand::Http(sub_cmd) => sub_cmd.execute().await?,
        }
        Ok(())
    }
}

fn verify_file(filename: &str) -> Result<String, &'static str> {
    // if input is "-" or file exists
    if filename == "-" || Path::new(filename).exists() {
        Ok(filename.into())
    } else {
        Err("input file not found")
    }
}

fn verify_path(path: &str) -> Result<PathBuf, &'static str> {
    let p = Path::new(path);
    if p.exists() && p.is_dir() {
        Ok(path.into())
    } else {
        Err("path not found or is not a directory")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_input_file() {
        assert_eq!(verify_file("-"), Ok("-".into()));
        assert_eq!(verify_file("Cargo.toml"), Ok("Cargo.toml".into()));
        assert_eq!(
            verify_file("non-existent-file"),
            Err("input file not found")
        );
    }
}
