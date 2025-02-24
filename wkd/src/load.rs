use std::time::SystemTime;

use bytes::Bytes;
use miette::Diagnostic;
use sequoia_openpgp::{self as openpgp, types::RevocationStatus};
use openpgp::{parse::Parse, Cert, policy::StandardPolicy};

use thiserror::Error;

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
}


/// Load a key from a byte array and return Ok(()) if successful
pub fn load_key(data: Bytes) -> Result<WkdKey, WkdLoadError> {
    let cert = match Cert::from_bytes(&data) {
        Ok(cert) => cert,
        Err(err) => {
            return Err(WkdLoadError::FailedToParseKey(err));
        }
    };

    let revocation_status = match cert.revocation_status(&StandardPolicy::new(), Some(SystemTime::now())) {
        RevocationStatus::Revoked(_) => "Revoked".to_string(),
        RevocationStatus::NotAsFarAsWeKnow => "Not as far as we know".to_string(),
        RevocationStatus::CouldBe(_) => "Revoked by third-party".to_string(),
    };

    Ok(WkdKey {
        fingerprint: cert.fingerprint().to_string(),
        revocation_status
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

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
    fn test_load_key_success() {
        let test_file_path = "../test_files/test_key";
        let key_bytes = fs::read(test_file_path).unwrap();
        let data = Bytes::from(key_bytes);
        let cert = load_key(data);
        assert!(cert.is_ok());
        let cert = cert.unwrap();
        assert_eq!(cert.fingerprint, "AC48BC1F029B6188D97E2D807C855DB4466DF0C6");
        assert_eq!(cert.revocation_status, "Not as far as we know");
    }
}
