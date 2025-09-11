use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub enum WkdMethodType {
    Direct,
    Advanced,
}
#[derive(Serialize, Deserialize)]
pub struct WkdResult {
    user_id: String,
    methods: Vec<WkdUriResult>,
}
#[derive(Serialize, Deserialize)]
pub struct WkdUriResult {
    uri: String,
    key: Option<WkdKey>,
    errors: Vec<WkdError>,
    method_type: WkdMethodType,
}
#[derive(Serialize, Deserialize)]
pub struct WkdError {
    name: String,
    message: String,
}
#[derive(Serialize, Deserialize)]
pub struct WkdKey {
    fingerprint: String,
    revocation_status: String,
    expiry: String,
    algorithm: String,
    randomart: String,
}

pub async fn get_wkd(user_id: &str) -> WkdResult {
    let wkd_uri = match wkd::uri::WkdUri::new(user_id) {
        Ok(wkd_uri) => wkd_uri,
        Err(err) => {
            return WkdResult {
                user_id: user_id.to_string(),
                methods: vec![
                    WkdUriResult {
                        uri: "".to_string(),
                        key: None,
                        errors: vec![WkdError::from(&err)],
                        method_type: WkdMethodType::Direct,
                    },
                    WkdUriResult {
                        uri: "".to_string(),
                        key: None,
                        errors: vec![WkdError::from(&err)],
                        method_type: WkdMethodType::Advanced,
                    },
                ],
            };
        }
    };

    let wkd_fetch = wkd::fetch::WkdFetch::fetch(&wkd_uri).await;
    let methods = vec![
        WkdUriResult::from(
            wkd_fetch.direct_method,
            wkd_uri.direct_uri,
            WkdMethodType::Direct,
        ),
        WkdUriResult::from(
            wkd_fetch.advanced_method,
            wkd_uri.advanced_uri,
            WkdMethodType::Advanced,
        ),
    ];
    WkdResult {
        user_id: user_id.to_string(),
        methods,
    }
}

impl WkdUriResult {
    pub fn from(
        wkd_fetch: wkd::fetch::WkdFetchUriResult,
        uri: impl std::string::ToString,
        method_type: WkdMethodType,
    ) -> Self {
        let key: Option<WkdKey> = match wkd_fetch.data {
            Some(data) => wkd::load::load_key(data).ok().map(WkdKey::from),
            None => None,
        };

        WkdUriResult {
            uri: uri.to_string(),
            key,
            errors: wkd_fetch.errors.iter().map(WkdError::from).collect(),
            method_type,
        }
    }
}

impl WkdError {
    pub fn from<Error: std::error::Error>(error: Error) -> Self {
        WkdError {
            name: format!("{error:?}"),
            message: format!("{error}"),
        }
    }
}

impl WkdKey {
    pub fn from(wkd_key: wkd::load::WkdKey) -> Self {
        WkdKey {
            fingerprint: wkd_key.fingerprint,
            revocation_status: wkd_key.revocation_status,
            expiry: wkd_key.expiry,
            algorithm: wkd_key.algorithm,
            randomart: wkd_key.randomart,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wkd_key_from() {
        let wkd_key = wkd::load::WkdKey {
            fingerprint: "fingerprint".to_string(),
            revocation_status: "revocation_status".to_string(),
            expiry: "expiry".to_string(),
            algorithm: "algorithm".to_string(),
            randomart: "randomart".to_string(),
        };
        let key = WkdKey::from(wkd_key);
        assert_eq!(key.fingerprint, "fingerprint");
        assert_eq!(key.revocation_status, "revocation_status");
        assert_eq!(key.expiry, "expiry");
        assert_eq!(key.algorithm, "algorithm");
        assert_eq!(key.randomart, "randomart");
    }

    #[test]
    fn test_wkd_error_from() {
        let error = wkd::fetch::WkdFetchError::AccessControlAllowOriginNotStar;
        let wkd_error = WkdError::from(error);
        assert_eq!(wkd_error.name, "AccessControlAllowOriginNotStar");
        assert_eq!(
            wkd_error.message,
            "Access-Control-Allow-Origin header is not set to '*'. This may cause issues with CORS"
        );
    }

    #[test]
    fn test_wkd_uri_result_from() {
        let wkd_fetch = wkd::fetch::WkdFetchUriResult {
            errors: vec![wkd::fetch::WkdFetchError::AccessControlAllowOriginNotStar],
            data: None,
        };
        let wkd_uri_result = WkdUriResult::from(wkd_fetch, "uri", WkdMethodType::Direct);
        assert!(wkd_uri_result.key.is_none());
        assert_eq!(wkd_uri_result.errors.len(), 1);
        assert_eq!(
            wkd_uri_result.errors[0].name,
            "AccessControlAllowOriginNotStar"
        );
        assert_eq!(
            wkd_uri_result.errors[0].message,
            "Access-Control-Allow-Origin header is not set to '*'. This may cause issues with CORS"
        );
    }

    #[tokio::test]
    async fn test_get_wkd() {
        let wkd_result = get_wkd("Joe.Doe@example.org").await;
        assert_eq!(wkd_result.user_id, "Joe.Doe@example.org");
        assert_eq!(
            wkd_result.methods.as_slice()[0].uri,
            "https://example.org/.well-known/openpgpkey/hu/iy9q119eutrkn8s1mk4r39qejnbu3n5q?l=Joe.Doe"
        );
        assert_eq!(
            wkd_result.methods.as_slice()[1].uri,
            "https://openpgpkey.example.org/.well-known/openpgpkey/example.org/hu/iy9q119eutrkn8s1mk4r39qejnbu3n5q?l=Joe.Doe"
        );
        assert!(wkd_result.methods.as_slice()[0].key.is_none());
        assert!(wkd_result.methods.as_slice()[1].key.is_none());
        assert_eq!(wkd_result.methods.as_slice()[1].errors.len(), 3);
        assert_eq!(wkd_result.methods.as_slice()[0].errors.len(), 3);
    }
}
