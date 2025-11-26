use sha1::{Digest, Sha1};
use std::fmt::Display;
use std::fmt::Formatter;

#[cfg(feature = "tracing")]
use tracing::{Level, event};

/// Direct Method URI conforming to <https://datatracker.ietf.org/doc/html/draft-koch-openpgp-webkey-service-19#section-3.1-10>
#[derive(Debug)]
pub struct DirectUri(String);

#[derive(Debug)]
/// Advanced Method URI conforming to <https://datatracker.ietf.org/doc/html/draft-koch-openpgp-webkey-service-19#section-3.1-5>
pub struct AdvancedUri(String);

#[derive(Debug)]
/// User Hash conforming to <https://datatracker.ietf.org/doc/html/draft-koch-openpgp-webkey-service-19#section-3.1-3>
pub struct UserHash(String);

use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Diagnostic, Debug, PartialEq)]
pub enum WkdUriError {
    #[cfg(test)]
    #[error("User hash must be 32 characters long")]
    #[diagnostic(
        code(wkd_uri::user_hash::from_string),
        url(
            "https://datatracker.ietf.org/doc/html/draft-koch-openpgp-webkey-service-19#section-3.1-3"
        )
    )]
    HashLengthError,

    #[cfg(test)]
    #[error("Invalid Z32 encoding")]
    #[diagnostic(
        code(wkd_uri::user_hash::from_string),
        url("https://philzimmermann.com/docs/human-oriented-base-32-encoding.txt")
    )]
    HashZ32EncodingError,

    #[error("User ID must be in the format '{{local_part}}@{{domain_part}}'")]
    #[diagnostic(
        code(wkd_uri::parse_email),
        url(
            "https://datatracker.ietf.org/doc/html/draft-koch-openpgp-webkey-service-19#section-3.1-2"
        )
    )]
    InvalidEmailError,
}

impl UserHash {
    #[cfg(test)]
    fn from_string(s: &str) -> Result<UserHash, WkdUriError> {
        if s.len() != 32 {
            return Err(WkdUriError::HashLengthError);
        }

        if z32::decode(s.as_bytes()).is_err() {
            return Err(WkdUriError::HashZ32EncodingError);
        }

        Ok(UserHash(s.to_string()))
    }

    fn new(local_part: &str) -> UserHash {
        let mut hasher = Sha1::new();
        let local_part = local_part.to_string().to_ascii_lowercase();
        hasher.update(local_part);
        let sha1_hash = hasher.finalize();
        let sha1_hash = z32::encode(&sha1_hash);
        assert!(sha1_hash.len() == 32);

        UserHash(sha1_hash)
    }
}

impl Display for UserHash {
    fn fmt(&self, f: &mut Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl Display for DirectUri {
    fn fmt(&self, f: &mut Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl Display for AdvancedUri {
    fn fmt(&self, f: &mut Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}

pub trait Uri<T> {
    const SCHEME: &str = "https://";
    const PATH: &str;
    const SUBDOMAIN: &str = "openpgpkey";

    fn new(domain_part: &str, local_part: &str, user_hash: &UserHash) -> Self;
}

impl Uri<DirectUri> for DirectUri {
    const PATH: &str = ".well-known/openpgpkey/hu";
    fn new(domain_part: &str, local_part: &str, user_hash: &UserHash) -> DirectUri {
        let scheme = Self::SCHEME;
        let path = Self::PATH;
        let hostname = domain_part;
        let uri = format!("{scheme}{hostname}/{path}/{user_hash}?l={local_part}");
        DirectUri(uri)
    }
}

impl Uri<AdvancedUri> for AdvancedUri {
    const PATH: &str = ".well-known/openpgpkey";

    fn new(domain_part: &str, local_part: &str, user_hash: &UserHash) -> AdvancedUri {
        let scheme = Self::SCHEME;
        let path = Self::PATH;
        let hostname = format!("{}.{domain_part}", Self::SUBDOMAIN);
        let uri = format!("{scheme}{hostname}/{path}/{domain_part}/hu/{user_hash}?l={local_part}");
        AdvancedUri(uri)
    }
}

fn parse_email(email: &str) -> Result<(&str, &str), WkdUriError> {
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return Err(WkdUriError::InvalidEmailError);
    }
    Ok((parts[0], parts[1]))
}

/// WkdUri struct that contains the domain_part, user_hash, local_part, advanced_uri and direct_uri
#[derive(Debug)]
pub struct WkdUri {
    /// The domain part of the email address
    pub domain_part: String,
    /// User Hash conforming to <https://datatracker.ietf.org/doc/html/draft-koch-openpgp-webkey-service-19#section-3.1-3>
    pub user_hash: UserHash,
    /// The local part of the email address
    pub local_part: String,
    /// Advanced Method URI conforming to <https://datatracker.ietf.org/doc/html/draft-koch-openpgp-webkey-service-19#section-3.1-5>
    pub advanced_uri: AdvancedUri,
    /// Direct Method URI conforming to <https://datatracker.ietf.org/doc/html/draft-koch-openpgp-webkey-service-19#section-3.1-10>
    pub direct_uri: DirectUri,
}

impl WkdUri {
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn new(user_id: &str) -> Result<WkdUri, WkdUriError> {
        let (local_part, domain_part) = parse_email(user_id)?;
        #[cfg(feature = "tracing")]
        event!(
            Level::TRACE,
            "Parsed email: local_part={}, domain_part={}",
            local_part,
            domain_part
        );
        let user_hash = UserHash::new(local_part);
        #[cfg(feature = "tracing")]
        event!(
            Level::TRACE,
            "Generated UserHash for local_part: {:?}",
            user_hash
        );

        let advanced_uri = AdvancedUri::new(domain_part, local_part, &user_hash);
        #[cfg(feature = "tracing")]
        event!(Level::TRACE, "Generated AdvancedUri: {:?}", advanced_uri);
        let direct_uri = DirectUri::new(domain_part, local_part, &user_hash);
        #[cfg(feature = "tracing")]
        event!(Level::TRACE, "Generated DirectUri: {:?}", direct_uri);

        Ok(WkdUri {
            domain_part: domain_part.to_string(),
            user_hash,
            local_part: local_part.to_string(),
            advanced_uri,
            direct_uri,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOMAIN_PART: &str = "example.org";
    const LOCAL_PART: &str = "Joe.Doe";
    const USER_HASH: &str = "iy9q119eutrkn8s1mk4r39qejnbu3n5q";
    const DIRECT_URI: &str =
        "https://example.org/.well-known/openpgpkey/hu/iy9q119eutrkn8s1mk4r39qejnbu3n5q?l=Joe.Doe";
    const ADVANCED_URI: &str = "https://openpgpkey.example.org/.well-known/openpgpkey/example.org/hu/iy9q119eutrkn8s1mk4r39qejnbu3n5q?l=Joe.Doe";

    #[test]
    fn generate_direct_uri() {
        let test_direct_uri = DirectUri::new(
            DOMAIN_PART,
            LOCAL_PART,
            &UserHash::from_string(USER_HASH).unwrap(),
        );
        assert_eq!(test_direct_uri.to_string(), DIRECT_URI.to_string());
    }

    #[test]
    fn generate_advanced_uri() {
        let test_advanced_uri = AdvancedUri::new(
            DOMAIN_PART,
            LOCAL_PART,
            &UserHash::from_string(USER_HASH).unwrap(),
        );
        assert_eq!(test_advanced_uri.to_string(), ADVANCED_URI.to_string());
    }

    #[test]
    fn user_hash_from_str_ok() {
        let test_user_hash = UserHash::from_string(USER_HASH);
        assert!(test_user_hash.is_ok());
        let test_user_hash = test_user_hash.unwrap();
        assert_eq!(test_user_hash.to_string().len(), 32);
        assert_eq!(test_user_hash.to_string(), USER_HASH.to_string());
    }

    #[test]
    fn user_hash_from_str_err_length() {
        let test_user_hash = UserHash::from_string("123");
        assert!(test_user_hash.is_err());
        assert_eq!(test_user_hash.unwrap_err(), WkdUriError::HashLengthError);
    }

    #[test]
    fn user_hash_from_str_err_z32_encoding() {
        let test_user_hash = UserHash::from_string("iy9q119eutrkn8s1mk4r39qejnbu3n5-");

        assert!(test_user_hash.is_err());
        assert_eq!(
            test_user_hash.unwrap_err(),
            WkdUriError::HashZ32EncodingError
        );
    }

    #[test]
    fn user_hash_new() {
        let test_user_hash = UserHash::new(LOCAL_PART);
        assert_eq!(test_user_hash.to_string().len(), 32);
        assert_eq!(test_user_hash.to_string(), USER_HASH.to_string());
    }

    #[test]
    fn user_hash_new_non_ascii() {
        let test_user_hash = UserHash::new("Grüße.Jürgen");
        assert_eq!(test_user_hash.to_string().len(), 32);
        assert_eq!(
            test_user_hash.to_string(),
            "izrmh5mqsi4zyh6njd4sxxh4g7xjrxq1"
        );
    }

    #[test]
    fn parse_email_ok() {
        let test_email = "test@example.org";
        let (local_part, domain_part) = parse_email(test_email).unwrap();
        assert_eq!(local_part, "test");
        assert_eq!(domain_part, "example.org");
    }

    #[test]
    fn parse_email_err() {
        let test_email = "test";
        let result = parse_email(test_email);
        assert!(result.is_err());
    }

    #[test]
    fn wkd_uri_new_invalid_email() {
        let test_wkd_uri = WkdUri::new("test");
        assert!(test_wkd_uri.is_err());
        assert_eq!(test_wkd_uri.unwrap_err(), WkdUriError::InvalidEmailError);
    }

    #[test]
    fn wkd_uri_new() {
        let test_wkd_uri = WkdUri::new(&format!("{LOCAL_PART}@{DOMAIN_PART}"));
        assert!(test_wkd_uri.is_ok());
        let test_wkd_uri = test_wkd_uri.unwrap();

        assert_eq!(test_wkd_uri.domain_part, DOMAIN_PART);
        assert_eq!(test_wkd_uri.local_part, LOCAL_PART);
        assert_eq!(test_wkd_uri.user_hash.to_string(), USER_HASH);
        assert_eq!(test_wkd_uri.advanced_uri.to_string(), ADVANCED_URI);
        assert_eq!(test_wkd_uri.direct_uri.to_string(), DIRECT_URI);
    }
}
