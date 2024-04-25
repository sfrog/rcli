use std::fs;

use clap::Parser;
use rcli::{
    process_base64_decode, process_base64_encode, process_csv, process_gen_pass,
    process_http_serve, process_text_generate, process_text_sign, process_text_verify,
    Base64SubCommand, HttpSubCommand, Opts, SubCommand, TextSignFormat, TextSubCommand,
};
use zxcvbn::zxcvbn;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
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
        SubCommand::GenPass(opts) => {
            let password = process_gen_pass(
                opts.length,
                opts.upper,
                opts.lower,
                opts.number,
                opts.symbol,
            )?;
            println!("{}", password);

            let estimate = zxcvbn(&password, &[])?;
            eprintln!("Estimated strength: {}", estimate.score());
        }
        SubCommand::Base64(sub_cmd) => match sub_cmd {
            Base64SubCommand::Encode(opts) => {
                let result = process_base64_encode(&opts.input, opts.format)?;
                println!("{}", result);
            }
            Base64SubCommand::Decode(opts) => {
                let result = process_base64_decode(&opts.input, opts.format)?;
                println!("{}", String::from_utf8(result)?);
            }
        },
        SubCommand::Text(sub_cmd) => match sub_cmd {
            TextSubCommand::Sign(opts) => {
                let signed = process_text_sign(&opts.input, &opts.key, opts.format)?;
                println!("{}", signed);
            }
            TextSubCommand::Verify(opts) => {
                let valid = process_text_verify(&opts.input, &opts.key, &opts.sig, opts.format)?;
                println!("{}", valid);
            }
            TextSubCommand::Generate(opts) => {
                let keys = process_text_generate(opts.format)?;
                match opts.format {
                    TextSignFormat::Blake3 => {
                        let name = opts.output.join("blake3.key");
                        fs::write(name, &keys[0])?;
                    }
                    TextSignFormat::Ed25519 => {
                        let name = opts.output;
                        fs::write(name.join("ed25519.sk"), &keys[0])?;
                        fs::write(name.join("ed25519.pk"), &keys[1])?;
                    }
                }
            }
        },
        SubCommand::Http(sub_cmd) => match sub_cmd {
            HttpSubCommand::Serve(opts) => {
                process_http_serve(opts.dir, opts.port).await?;
            }
        },
    }

    Ok(())
}
