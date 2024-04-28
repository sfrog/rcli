use chrono::Utc;
use core::fmt;
use humantime::parse_duration;
use std::{fs, path::Path};

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::KeyLoader;

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub aud: String,
    pub exp: usize,
}

struct JwtKey {
    key: [u8; 32],
}

impl KeyLoader for JwtKey {
    fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let key = fs::read(path)?;
        let key = &key[..32];
        let key = key.try_into()?;
        Ok(Self { key })
    }
}

fn str_to_timestamp(str: &str) -> anyhow::Result<i64> {
    let seconds = parse_duration(str)?.as_secs();
    let now = Utc::now().timestamp();

    let timestamp = now + seconds as i64;

    Ok(timestamp)
}

pub fn process_jwt_sign(key: &str, sub: &str, aud: &str, exp: &str) -> anyhow::Result<String> {
    let key = JwtKey::load(key)?;

    let timestamp = str_to_timestamp(exp)?;

    let claims = JwtClaims {
        sub: sub.to_string(),
        aud: aud.to_string(),
        exp: timestamp as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(key.key.as_ref()),
    )?;
    Ok(token)
}

pub fn process_jwt_verify(key: &str, token: &str) -> anyhow::Result<JwtClaims> {
    let key = JwtKey::load(key)?;

    let mut validation = Validation::default();
    validation.validate_aud = false;

    let token_message = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(key.key.as_ref()),
        &validation,
    )?;

    let claims = token_message.claims;
    Ok(claims)
}

impl fmt::Display for JwtClaims {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_jwt_sign() -> anyhow::Result<()> {
        let key = "fixtures/jwt.key";
        let sub = "test";
        let aud = "test";
        let exp = "1d";

        let token = process_jwt_sign(key, sub, aud, exp)?;
        let claims = process_jwt_verify(key, &token)?;

        assert!(claims.sub == sub);
        assert!(claims.aud == aud);
        Ok(())
    }
}
