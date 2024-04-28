use super::{verify_file, verify_path};
use crate::{process, CmdExecutor};
use clap::Parser;
use enum_dispatch::enum_dispatch;
use std::{fmt, path::PathBuf, str::FromStr};
use tokio::fs;

#[derive(Parser, Debug)]
#[enum_dispatch(CmdExecutor)]
pub enum TextSubCommand {
    #[command(about = "Sign a message with a private/shared key")]
    Sign(TextSignOpts),
    #[command(about = "Verify a signed message")]
    Verify(TextVerifyOpts),
    #[command(about = "Generate a key pair")]
    Generate(GenerateOpts),
    #[command(about = "Encrypt a message")]
    Encrypt(EncryptOpts),
    #[command(about = "Decrypt a message")]
    Decrypt(DecryptOpts),
}

#[derive(Parser, Debug)]
pub struct TextSignOpts {
    #[arg(short, long, value_parser = verify_file, default_value = "-")]
    pub input: String,
    #[arg(short, long, value_parser = verify_file)]
    pub key: String,
    #[arg(long, value_parser = parse_sign_format, default_value = "blake3")]
    pub format: TextSignFormat,
}

impl CmdExecutor for TextSignOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let signed = process::process_text_sign(&self.input, &self.key, self.format)?;
        println!("{}", signed);
        Ok(())
    }
}

#[derive(Parser, Debug)]
pub struct TextVerifyOpts {
    #[arg(short, long, value_parser = verify_file, default_value = "-")]
    pub input: String,
    #[arg(short, long, value_parser = verify_file)]
    pub key: String,
    #[arg(long, value_parser = parse_sign_format, default_value = "blake3")]
    pub format: TextSignFormat,
    #[arg(short, long)]
    pub sig: String,
}

impl CmdExecutor for TextVerifyOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let valid = process::process_text_verify(&self.input, &self.key, &self.sig, self.format)?;
        println!("{}", valid);
        Ok(())
    }
}

#[derive(Parser, Debug)]
pub struct GenerateOpts {
    #[arg(short, long, value_parser = parse_sign_format, default_value = "blake3")]
    pub format: TextSignFormat,
    #[arg(short, long, value_parser = verify_path)]
    pub output: PathBuf,
}

impl CmdExecutor for GenerateOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let keys = process::process_text_generate(self.format)?;
        match self.format {
            TextSignFormat::Blake3 => {
                let name = self.output.join("blake3.key");
                fs::write(name, &keys[0]).await?;
            }
            TextSignFormat::Ed25519 => {
                let name = self.output;
                fs::write(name.join("ed25519.sk"), &keys[0]).await?;
                fs::write(name.join("ed25519.pk"), &keys[1]).await?;
            }
            TextSignFormat::Chacha => {
                let name = self.output;
                fs::write(name.join("chacha.key"), &keys[0]).await?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TextSignFormat {
    Blake3,
    Ed25519,
    Chacha,
}

pub fn parse_sign_format(format: &str) -> Result<TextSignFormat, anyhow::Error> {
    format.parse()
}

impl FromStr for TextSignFormat {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "blake3" => Ok(TextSignFormat::Blake3),
            "ed25519" => Ok(TextSignFormat::Ed25519),
            "chacha" => Ok(TextSignFormat::Chacha),
            v => anyhow::bail!("Unsupported format: {}", v),
        }
    }
}

impl From<TextSignFormat> for &'static str {
    fn from(format: TextSignFormat) -> Self {
        match format {
            TextSignFormat::Blake3 => "blake3",
            TextSignFormat::Ed25519 => "ed25519",
            TextSignFormat::Chacha => "chacha",
        }
    }
}

impl fmt::Display for TextSignFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Into::<&'static str>::into(*self))
    }
}

#[derive(Parser, Debug)]
pub struct EncryptOpts {
    #[arg(short, long, value_parser = verify_file, default_value = "-")]
    pub input: String,
    #[arg(short, long, value_parser = verify_file)]
    pub key: String,
}

impl CmdExecutor for EncryptOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let encrypted = process::process_text_encrypt(&self.input, &self.key)?;
        println!("{}", encrypted);
        Ok(())
    }
}

#[derive(Parser, Debug)]
pub struct DecryptOpts {
    #[arg(short, long, value_parser = verify_file, default_value = "-")]
    pub input: String,
    #[arg(short, long, value_parser = verify_file)]
    pub key: String,
}

impl CmdExecutor for DecryptOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let decrypted = process::process_text_decrypt(&self.input, &self.key)?;
        println!("{}", decrypted);
        Ok(())
    }
}
