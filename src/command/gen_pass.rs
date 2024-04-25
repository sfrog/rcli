use clap::Parser;
use zxcvbn::zxcvbn;

use crate::{process, CmdExecutor};

#[derive(Parser, Debug)]
pub struct GenPassOpts {
    #[arg(short, long, default_value_t = 16)]
    pub length: u8,

    #[arg(long, default_value_t = true)]
    pub upper: bool,

    #[arg(long, default_value_t = true)]
    pub lower: bool,

    #[arg(long, default_value_t = true)]
    pub number: bool,

    #[arg(long, default_value_t = true)]
    pub symbol: bool,
}

impl CmdExecutor for GenPassOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let password = process::process_gen_pass(
            self.length,
            self.upper,
            self.lower,
            self.number,
            self.symbol,
        )?;
        println!("{}", password);

        let estimate = zxcvbn(&password, &[])?;
        eprintln!("Estimated strength: {}", estimate.score());
        Ok(())
    }
}
