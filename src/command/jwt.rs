use crate::{process, CmdExecutor};
use clap::Parser;
use enum_dispatch::enum_dispatch;

use super::verify_file;

#[derive(Parser, Debug)]
#[enum_dispatch(CmdExecutor)]
pub enum JwtSubCommand {
    #[command(about = "Get a JWT token")]
    Sign(JwtSignOpts),
    #[command(about = "Verify a JWT token")]
    Verify(JwtVerifyOpts),
}

#[derive(Parser, Debug)]
pub struct JwtSignOpts {
    #[arg(short, long, value_parser = verify_file)]
    pub key: String,
    #[arg(long)]
    pub sub: String,
    #[arg(long)]
    pub aud: String,
    #[arg(long, default_value = "14d")]
    pub exp: String,
}

impl CmdExecutor for JwtSignOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let token = process::process_jwt_sign(&self.key, &self.sub, &self.aud, &self.exp)?;
        println!("{}", token);
        Ok(())
    }
}

#[derive(Parser, Debug)]
pub struct JwtVerifyOpts {
    #[arg(short, long, value_parser = verify_file)]
    pub key: String,
    #[arg(short, long)]
    pub token: String,
}

impl CmdExecutor for JwtVerifyOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let valid = process::process_jwt_verify(&self.key, &self.token)?;
        println!("{}", valid);
        Ok(())
    }
}
