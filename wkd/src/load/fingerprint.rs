use std::fmt;

use pgp::{
    composed::SignedPublicKey,
    types::{KeyDetails, KeyVersion},
};

pub struct Fingerprint {
    pub fingerprint: Vec<u8>,
    pub algorithm: String,
}

impl fmt::Display for Fingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fingerprint = hex::encode(self.fingerprint.clone());
        let fingerprint = fingerprint.to_ascii_uppercase();

        write!(f, "{}", fingerprint)
    }
}

impl Fingerprint {
    pub fn as_bytes(&self) -> &[u8] {
        self.fingerprint.as_slice()
    }

    pub fn new(pub_key: &SignedPublicKey) -> Self {
        let fingerprint = pub_key.fingerprint().as_bytes().to_vec();

        let algorithm = match pub_key.version() {
            KeyVersion::V2 | KeyVersion::V3 => "MD5",
            KeyVersion::V4 => "SHA1",
            KeyVersion::V5 | KeyVersion::V6 => "SHA256",
            KeyVersion::Other(_) => "NONE",
        };

        let algorithm = algorithm.to_string();

        Fingerprint {
            fingerprint,
            algorithm,
        }
    }
}
