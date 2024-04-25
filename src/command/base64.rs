use std::{fmt, str::FromStr};

use clap::Parser;

use crate::{process, CmdExecutor};

use super::verify_file;

#[derive(Parser, Debug)]
pub enum Base64SubCommand {
    #[command(name = "encode", about = "Encode a file to base64")]
    Encode(EncodeOpts),
    #[command(name = "decode", about = "Decode a base64 file")]
    Decode(DecodeOpts),
}

impl CmdExecutor for Base64SubCommand {
    async fn execute(self) -> anyhow::Result<()> {
        match self {
            Base64SubCommand::Encode(opts) => opts.execute().await?,
            Base64SubCommand::Decode(opts) => opts.execute().await?,
        }
        Ok(())
    }
}

#[derive(Parser, Debug)]
pub struct EncodeOpts {
    #[arg(short, long, value_parser = verify_file, default_value = "-")]
    pub input: String,
    #[arg(long, value_parser = parse_base64_format, default_value = "standard")]
    pub format: Base64Format,
}

impl CmdExecutor for EncodeOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let result = process::process_base64_encode(&self.input, self.format)?;
        println!("{}", result);
        Ok(())
    }
}

#[derive(Parser, Debug)]
pub struct DecodeOpts {
    #[arg(short, long, value_parser = verify_file, default_value = "-")]
    pub input: String,
    #[arg(long, value_parser = parse_base64_format, default_value = "standard")]
    pub format: Base64Format,
}

impl CmdExecutor for DecodeOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let result = process::process_base64_decode(&self.input, self.format)?;
        println!("{}", String::from_utf8(result)?);
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Base64Format {
    Standard,
    UrlSafe,
}

pub fn parse_base64_format(format: &str) -> Result<Base64Format, anyhow::Error> {
    format.parse()
}

impl FromStr for Base64Format {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "standard" => Ok(Base64Format::Standard),
            "urlsafe" => Ok(Base64Format::UrlSafe),
            v => anyhow::bail!("Unsupported format: {}", v),
        }
    }
}

impl From<Base64Format> for &'static str {
    fn from(format: Base64Format) -> Self {
        match format {
            Base64Format::Standard => "standard",
            Base64Format::UrlSafe => "urlsafe",
        }
    }
}

impl fmt::Display for Base64Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Into::<&'static str>::into(*self))
    }
}
