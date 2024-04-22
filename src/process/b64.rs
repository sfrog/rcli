use std::{fs::File, io::Read};

use base64::{engine::GeneralPurpose, prelude::*};

use crate::command::Base64Format;

pub fn process_base64_encode(input: &str, format: Base64Format) -> anyhow::Result<()> {
    let mut reader = get_reader(input);

    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;

    let engine = get_engine(format);

    let encode = engine.encode(buf);

    println!("{}", encode);

    Ok(())
}

pub fn process_base64_decode(input: &str, format: Base64Format) -> anyhow::Result<()> {
    let mut reader = get_reader(input);

    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    let buf = buf.trim();

    let engine = get_engine(format);

    let decode = engine.decode(buf)?;
    println!("{}", String::from_utf8(decode)?);
    Ok(())
}

fn get_reader(input: &str) -> Box<dyn Read> {
    if input == "-" {
        Box::new(std::io::stdin())
    } else {
        Box::new(File::open(input).expect("File not found"))
    }
}

fn get_engine(format: Base64Format) -> GeneralPurpose {
    match format {
        Base64Format::Standard => BASE64_STANDARD,
        Base64Format::UrlSafe => BASE64_URL_SAFE_NO_PAD,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_base64_encode_standard() {
        let input = "fixtures/base64/b64_original.txt";
        let format = Base64Format::Standard;
        assert!(process_base64_encode(input, format).is_ok());
    }

    #[test]
    fn test_process_base64_encode_urlsafe() {
        let input = "fixtures/base64/b64_original.txt";
        let format = Base64Format::UrlSafe;
        assert!(process_base64_encode(input, format).is_ok());
    }

    #[test]
    fn test_process_base64_decode_standard() {
        let input = "fixtures/base64/b64_standard.txt";
        let format = Base64Format::Standard;
        assert!(process_base64_decode(input, format).is_ok());
    }

    #[test]
    fn test_process_base64_decode_urlsafe() {
        let input = "fixtures/base64/b64_urlsafe.txt";
        let format = Base64Format::UrlSafe;
        assert!(process_base64_decode(input, format).is_ok());
    }
}
