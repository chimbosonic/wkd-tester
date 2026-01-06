use bytes::Bytes;
use chrono::{self};
use miette::Diagnostic;
use pgp::composed::{Deserializable, SignedPublicKey};
use pgp::types::{KeyDetails, PublicKeyTrait};
use thiserror::Error;

mod fingerprint;
mod randomart;
use fingerprint::Fingerprint;
use randomart::generate_randomart;

#[derive(Error, Diagnostic, Debug)]
pub enum WkdLoadError {
    #[error("Failed to parse key")]
    #[diagnostic(code(wkd_fetch))]
    FailedToParseKey(#[from] anyhow::Error),
}

#[derive(Debug)]
pub struct WkdKey {
    pub fingerprint: String,
    pub revocation_status: String,
    pub expiry: String,
    pub algorithm: String,
    pub randomart: String,
}

/// https://github.com/rpgp/rpgp/commit/0f58ea1cec37ca271282917d8df81fcf599f365f removed expires_at from SignedPublicKey
/// so we re-implement it here following the same logic as before.
fn expires_at(key: &SignedPublicKey) -> Option<chrono::DateTime<chrono::Utc>> {
    let expiration = key
        .details
        .users
        .iter()
        .flat_map(|user| &user.signatures)
        .filter_map(|sig| sig.key_expiration_time())
        .max()
        .cloned()?;

    Some(*key.primary_key.created_at() + expiration)
}

#[cfg_attr(feature = "tracing", tracing::instrument)]
pub fn load_key(data: Bytes) -> Result<WkdKey, WkdLoadError> {
    let pub_key = match SignedPublicKey::from_bytes(std::io::Cursor::new(data)) {
        Ok(key) => key,
        Err(err) => {
            return Err(WkdLoadError::FailedToParseKey(err.into()));
        }
    };

    let fingerprint = Fingerprint::new(&pub_key);

    let algorithm = format!("{:?}", pub_key.algorithm());

    let randomart = generate_randomart(&fingerprint, &algorithm).unwrap();

    let fingerprint = fingerprint.to_string();

    let revocation_status = match pub_key.verify() {
        Err(reason) => format!("Revoked: {}", reason),
        Ok(_) => "Not as far as we know".to_string(),
    };

    let expiry = match expires_at(&pub_key) {
        Some(date) => {
            if date < chrono::Utc::now() {
                format!("Expired on {}", date)
            } else {
                format!("Expires on {}", date)
            }
        }
        None => "No expiry date set".to_string(),
    };

    Ok(WkdKey {
        fingerprint,
        revocation_status,
        expiry,
        algorithm,
        randomart,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::load::WkdLoadError;

    use super::*;

    #[test]
    fn test_load_key_fail() {
        let data = Bytes::from("Hello, World!");
        let cert = load_key(data);
        assert!(cert.is_err());
        let cert = cert.unwrap_err();
        assert!(matches!(cert, WkdLoadError::FailedToParseKey(_)));
    }

    #[test]
    fn test_load_expired_key() {
        let test_file_path = "../test_files/test_expired_key";
        let key_bytes = fs::read(test_file_path).unwrap();
        let data = Bytes::from(key_bytes);
        let cert = load_key(data);
        assert!(cert.is_ok());
        let cert = cert.unwrap();
        assert_eq!(
            cert.fingerprint.to_string(),
            "A03351F7677A6D0B94F224A636CB3789EAC25E50"
        );
        assert_eq!(cert.revocation_status, "Not as far as we know");
        assert_eq!(cert.expiry, "Expired on 2021-08-26 15:38:21 UTC");
    }

    #[test]
    fn test_load_key_success() {
        let test_file_path = "../test_files/test_key";
        let key_bytes = fs::read(test_file_path).unwrap();
        let data = Bytes::from(key_bytes);
        let cert = load_key(data);
        assert!(cert.is_ok());
        let cert = cert.unwrap();
        assert_eq!(
            cert.fingerprint.to_string(),
            "AC48BC1F029B6188D97E2D807C855DB4466DF0C6"
        );
        assert_eq!(cert.expiry, "Expires on 2037-11-12 12:15:56 UTC");

        assert_eq!(
            cert.randomart,
            "+------[RSA]------+\n|      .=o        |\n|    o o +o       |\n|   . o o.E       |\n|o= .. ...        |\n|=.*.o   S        |\n| o.B + .         |\n|  + * +          |\n|   . + .         |\n|      .          |\n+-----[SHA1]------+"
        );
    }
}
