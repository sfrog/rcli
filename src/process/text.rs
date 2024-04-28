use crate::{command::TextSignFormat, get_reader, process_gen_pass};
use anyhow::{anyhow, Ok, Result};
use base64::prelude::*;
use chacha20poly1305::{
    aead::{generic_array::GenericArray, Aead, AeadCore, KeyInit, OsRng},
    ChaCha20Poly1305,
};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use std::{fs, io::Read, path::Path};

pub trait TextSign {
    /// Sign the input data
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>>;
}

pub trait TextVerify {
    /// Verify the input data
    fn verify(&self, reader: impl Read, sig: &[u8]) -> Result<bool>;
}

pub trait KeyLoader {
    /// Load the key from the given path
    fn load(path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized;
}

pub trait KeyGenerator {
    fn generate() -> Result<Vec<Vec<u8>>>;
}

pub trait TextEncrypt {
    fn encrypt(&self, reader: &mut dyn Read) -> Result<Vec<u8>>;
}

pub trait TextDecrypt {
    fn decrypt(&self, reader: &mut dyn Read) -> Result<Vec<u8>>;
}

pub struct Blake3 {
    key: [u8; 32],
}

pub struct Ed25519Signer {
    key: SigningKey,
}

pub struct Ed25519Verifier {
    key: VerifyingKey,
}

pub struct ChaCha20 {
    key: [u8; 32],
}

impl TextSign for Blake3 {
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;

        let hash = blake3::keyed_hash(&self.key, &buf);
        Ok(hash.as_bytes().to_vec())
    }
}

impl TextVerify for Blake3 {
    fn verify(&self, mut reader: impl Read, sig: &[u8]) -> Result<bool> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;

        let hash = blake3::keyed_hash(&self.key, &buf);
        Ok(hash.as_bytes() == sig)
    }
}

impl KeyLoader for Blake3 {
    fn load(path: impl AsRef<Path>) -> Result<Self> {
        let key = fs::read(path)?;
        let key = &key[..32];
        let key = key.try_into().unwrap();
        Ok(Self { key })
    }
}

impl KeyGenerator for Blake3 {
    fn generate() -> Result<Vec<Vec<u8>>> {
        let key = process_gen_pass(32, true, true, true, true)?;
        let key = key.as_bytes().to_vec();
        Ok(vec![key])
    }
}

impl TextSign for Ed25519Signer {
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;

        let sig = self.key.sign(&buf);
        Ok(sig.to_bytes().to_vec())
    }
}

impl KeyLoader for Ed25519Signer {
    fn load(path: impl AsRef<Path>) -> Result<Self> {
        let key = fs::read(path)?;
        let key = &key[..32];
        let key = key.try_into().unwrap();
        let key = SigningKey::from_bytes(&key);
        Ok(Self { key })
    }
}

impl KeyGenerator for Ed25519Signer {
    fn generate() -> Result<Vec<Vec<u8>>> {
        let mut csprng = OsRng;
        let sk = SigningKey::generate(&mut csprng);
        let pk = sk.verifying_key().to_bytes().to_vec();
        let sk = sk.to_bytes().to_vec();
        Ok(vec![sk, pk])
    }
}

impl TextVerify for Ed25519Verifier {
    fn verify(&self, mut reader: impl Read, sig: &[u8]) -> Result<bool> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;

        let sig = Signature::from_bytes(sig.try_into()?);
        Ok(self.key.verify(&buf, &sig).is_ok())
    }
}

impl KeyLoader for Ed25519Verifier {
    fn load(path: impl AsRef<Path>) -> Result<Self> {
        let key = fs::read(path)?;
        let key = &key[..32];
        let key = key.try_into().unwrap();
        let key = VerifyingKey::from_bytes(&key)?;
        Ok(Self { key })
    }
}

impl KeyGenerator for ChaCha20 {
    fn generate() -> Result<Vec<Vec<u8>>> {
        let key = process_gen_pass(32, true, true, true, false)?;
        let key = key.as_bytes().to_vec();
        Ok(vec![key])
    }
}

impl KeyLoader for ChaCha20 {
    fn load(path: impl AsRef<Path>) -> Result<Self> {
        let key = fs::read(path)?;
        let key = &key[..32];
        let key = key.try_into().unwrap();
        Ok(Self { key })
    }
}

impl TextEncrypt for ChaCha20 {
    fn encrypt(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;

        let cipher = ChaCha20Poly1305::new_from_slice(&self.key)?;
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        let cipher_text = cipher
            .encrypt(&nonce, buf.as_ref())
            .map_err(|_| anyhow!("Encryption failed"))?;
        let mut result = nonce.to_vec();
        result.extend_from_slice(&cipher_text);
        Ok(result)
    }
}

impl TextDecrypt for ChaCha20 {
    fn decrypt(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;

        let cipher = ChaCha20Poly1305::new_from_slice(&self.key)?;
        let nonce: &GenericArray<u8, _> = GenericArray::from_slice(&buf[..12]);
        let plain = cipher
            .decrypt(nonce, &buf[12..])
            .map_err(|_| anyhow!("Decryption failed"))?;
        Ok(plain)
    }
}

pub fn process_text_sign(input: &str, key: &str, format: TextSignFormat) -> Result<String> {
    let mut reader = get_reader(input)?;

    let signed = match format {
        TextSignFormat::Blake3 => {
            let signer = Blake3::load(key)?;
            signer.sign(&mut reader)?
        }
        TextSignFormat::Ed25519 => {
            let signer = Ed25519Signer::load(key)?;
            signer.sign(&mut reader)?
        }
        _ => return Err(anyhow!("Unsupported format")),
    };

    let base64 = BASE64_URL_SAFE_NO_PAD.encode(signed);

    Ok(base64)
}

pub fn process_text_verify(
    input: &str,
    key: &str,
    sig: &str,
    format: TextSignFormat,
) -> Result<bool> {
    let mut reader = get_reader(input)?;

    let valid = match format {
        TextSignFormat::Blake3 => {
            let verifier = Blake3::load(key)?;
            verifier.verify(&mut reader, &BASE64_URL_SAFE_NO_PAD.decode(sig)?)?
        }
        TextSignFormat::Ed25519 => {
            let verifier = Ed25519Verifier::load(key)?;
            verifier.verify(&mut reader, &BASE64_URL_SAFE_NO_PAD.decode(sig)?)?
        }
        _ => return Err(anyhow!("Unsupported format")),
    };

    Ok(valid)
}

pub fn process_text_generate(format: TextSignFormat) -> Result<Vec<Vec<u8>>> {
    match format {
        TextSignFormat::Blake3 => Blake3::generate(),
        TextSignFormat::Ed25519 => Ed25519Signer::generate(),
        TextSignFormat::Chacha => ChaCha20::generate(),
    }
}

pub fn process_text_encrypt(input: &str, key: &str) -> Result<String> {
    let mut reader = get_reader(input)?;

    let chacha = ChaCha20::load(key)?;
    let result = chacha.encrypt(&mut reader)?;

    Ok(BASE64_URL_SAFE_NO_PAD.encode(result))
}

pub fn process_text_decrypt(input: &str, key: &str) -> Result<String> {
    let mut reader = get_reader(input)?;
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    let nonce_and_cipher = BASE64_URL_SAFE_NO_PAD.decode(buf)?;

    let chacha = ChaCha20::load(key)?;
    let plaintext = chacha.decrypt(&mut &nonce_and_cipher[..])?;

    Ok(String::from_utf8(plaintext)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake3_sign_verify() -> Result<()> {
        let signer = Blake3::load("fixtures/blake3.key")?;

        let data = b"hello";
        let sig = signer.sign(&mut &data[..])?;
        assert!(signer.verify(&mut &data[..], &sig)?);
        Ok(())
    }

    #[test]
    fn test_ed25519_sign_verify() -> Result<()> {
        let signer = Ed25519Signer::load("fixtures/ed25519.sk")?;
        let verifier = Ed25519Verifier::load("fixtures/ed25519.pk")?;

        let data = b"hello world!";
        let sig = signer.sign(&mut &data[..])?;
        assert!(verifier.verify(&mut &data[..], &sig)?);
        Ok(())
    }

    #[test]
    fn test_chacha20_encrypt_decrypt() -> Result<()> {
        let chacha = ChaCha20::load("fixtures/chacha.key")?;
        let data = b"hello world!";
        let cipher = chacha.encrypt(&mut &data[..])?;
        let plain = chacha.decrypt(&mut &cipher[..])?;
        assert_eq!(data.to_vec(), plain);
        Ok(())
    }
}
