use bytes::Bytes;
use miette::Diagnostic;
use sequoia_openpgp::{parse::Parse, Cert};
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
pub enum WkdLoadError {
    #[error("Failed to parse key")]
    #[diagnostic(code(wkd_fetch))]
    FailedToParseKey(#[from] anyhow::Error),
}

/// Load a key from a byte array and return Ok(()) if successful
pub fn load_key(data: Bytes) -> Result<String, WkdLoadError> {
    let cert = match Cert::from_bytes(&data) {
        Ok(cert) => cert,
        Err(err) => {
            return Err(WkdLoadError::FailedToParseKey(err));
        }
    };

    Ok(cert.fingerprint().to_string())
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
    }
}
