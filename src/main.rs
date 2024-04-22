use clap::Parser;
use rcli::{
    process_base64_decode, process_base64_encode, process_csv, process_gen_pass, Base64SubCommand,
    Opts, SubCommand,
};

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();
    match opts.cmd {
        SubCommand::Csv(opts) => {
            let output = if let Some(output) = opts.output {
                output.clone()
            } else {
                format!("output.{}", opts.format)
            };
            process_csv(&opts.input, output, opts.format)?;
        }
        SubCommand::GenPass(opts) => process_gen_pass(
            opts.length,
            opts.upper,
            opts.lower,
            opts.number,
            opts.symbol,
        )?,
        SubCommand::Base64(sub_cmd) => match sub_cmd {
            Base64SubCommand::Encode(opts) => process_base64_encode(&opts.input, opts.format)?,
            Base64SubCommand::Decode(opts) => process_base64_decode(&opts.input, opts.format)?,
        },
    }

    Ok(())
}
