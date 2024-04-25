use crate::{process, CmdExecutor};

use super::verify_path;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub enum HttpSubCommand {
    #[command(about = "Serve a directory over HTTP")]
    Serve(HttpServeOpts),
}

impl CmdExecutor for HttpSubCommand {
    async fn execute(self) -> anyhow::Result<()> {
        match self {
            HttpSubCommand::Serve(opts) => opts.execute().await?,
        }
        Ok(())
    }
}

#[derive(Parser, Debug)]
pub struct HttpServeOpts {
    #[arg(short, long, default_value = ".", value_parser = verify_path)]
    pub dir: PathBuf,
    #[arg(short, long, default_value_t = 8080)]
    pub port: u16,
}

impl CmdExecutor for HttpServeOpts {
    async fn execute(self) -> anyhow::Result<()> {
        process::process_http_serve(self.dir, self.port).await?;

        Ok(())
    }
}
