use super::uri::{Uri, WkdUri};
use bytes::Bytes;
use reqwest::Url;

use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
pub enum WkdFetchError {
    #[error("WKD URI provided is not a valid URL")]
    #[diagnostic(code(wkd_fetch))]
    WkdUriNotValidUrl(#[from] url::ParseError),

    #[error("Failed to fetch given URL")]
    #[diagnostic(code(wkd_fetch))]
    FailedToFetchUrl(#[from] reqwest::Error),

    #[error(
        "Content-Type header is not set to 'application/octet-stream'. This may cause issues with parsing"
    )]
    #[diagnostic(severity(Warning), code(wkd_fetch))]
    ContentTypeNotOctetStream,

    #[error(
        "Access-Control-Allow-Origin header is not set to '*'. This may cause issues with CORS"
    )]
    #[diagnostic(severity(Warning), code(wkd_fetch))]
    AccessControlAllowOriginNotStar,

    #[error("Error whilst extracting body from response")]
    #[diagnostic(code(wkd_fetch))]
    NoDataReturned,

    #[error("Status code is not 200")]
    #[diagnostic(code(wkd_fetch))]
    StatusNot200(u16),

    #[error("Failed existence cheack with HEAD Method")]
    #[diagnostic(code(wkd_fetch))]
    FailedHeadMethod,

    #[error("Well-Known Path shouldn't have a index")]
    #[diagnostic(severity(Warning), code(wkd_fetch))]
    WkdPathShouldNotHaveIndex,

    #[error("Policy file not found")]
    #[diagnostic(severity(Warning), code(wkd_fetch))]
    WkdPolicyFileNotFound,

    #[error("Could not generate policy file path from URL")]
    #[diagnostic(severity(Warning), code(wkd_fetch))]
    WkdPolicyFilePathGenerationFailed,
}

pub struct WkdFetch {
    pub direct_method: WkdFetchUriResult,
    pub advanced_method: WkdFetchUriResult,
}

impl WkdFetch {
    pub async fn fetch(wkd_uri: &WkdUri) -> WkdFetch {
        let direct_method = fetch_uri(&wkd_uri.direct_uri).await;
        let advanced_method = fetch_uri(&wkd_uri.advanced_uri).await;

        WkdFetch {
            direct_method,
            advanced_method,
        }
    }
}

#[derive(Debug)]
pub struct WkdFetchUriResult {
    pub errors: Vec<WkdFetchError>,
    pub data: Option<Bytes>,
}

fn trim_uri(url: &str) -> &str {
    if let Some(pos) = url.rfind('/') {
        &url[..=pos]
    } else {
        url
    }
}

fn get_policy_url(url: &str) -> Option<String> {
    url.rfind("/hu/")
        .map(|pos| format!("{}policy", &url[..=pos]))
}

async fn check_head_method(client: &reqwest::Client, url: &str) -> Result<(), WkdFetchError> {
    if let Ok(response) = client.head(url).send().await
        && response.status().as_u16() == 200
    {
        return Ok(());
    }

    Err(WkdFetchError::FailedHeadMethod)
}

async fn check_for_indexing(client: &reqwest::Client, url: &str) -> Result<(), WkdFetchError> {
    let index_url = trim_uri(url);
    if let Ok(response) = client.get(index_url).send().await
        && response.status().as_u16() == 200
    {
        return Err(WkdFetchError::WkdPathShouldNotHaveIndex);
    }

    Ok(())
}

async fn check_policy_file(client: &reqwest::Client, url: &str) -> Result<(), WkdFetchError> {
    let policy_url = match get_policy_url(url) {
        Some(policy_url) => policy_url,
        None => return Err(WkdFetchError::WkdPolicyFilePathGenerationFailed),
    };

    if let Ok(response) = client.get(&policy_url).send().await
        && response.status().as_u16() == 200
    {
        return Ok(());
    }

    Err(WkdFetchError::WkdPolicyFileNotFound)
}

async fn fetch_uri<T>(
    uri: &(impl Uri<T> + std::fmt::Debug + std::string::ToString),
) -> WkdFetchUriResult {
    let mut result = WkdFetchUriResult {
        errors: Vec::new(),
        data: None,
    };

    let url = match Url::parse(&uri.to_string()) {
        Ok(url) => url,
        Err(err) => {
            result.errors.push(WkdFetchError::WkdUriNotValidUrl(err));
            return result;
        }
    };

    let client = reqwest::Client::new();

    if let Err(err) = check_head_method(&client, url.as_str()).await {
        result.errors.push(err);
    }

    if let Err(err) = check_for_indexing(&client, url.as_str()).await {
        result.errors.push(err);
    }

    if let Err(err) = check_policy_file(&client, url.as_str()).await {
        result.errors.push(err);
    }

    let response = match client.get(url).send().await {
        Ok(response) => response,
        Err(err) => {
            result.errors.push(WkdFetchError::FailedToFetchUrl(err));
            return result;
        }
    };

    let status = response.status().as_u16();
    if status != 200 {
        result.errors.push(WkdFetchError::StatusNot200(status));
        return result;
    }

    if let Some(header_value) = response.headers().get("content-type")
        && header_value != "application/octet-stream"
    {
        result.errors.push(WkdFetchError::ContentTypeNotOctetStream);
    }

    if let Some(header_value) = response.headers().get("access-control-allow-origin")
        && header_value != "*"
    {
        result
            .errors
            .push(WkdFetchError::AccessControlAllowOriginNotStar);
    }

    let data = match response.bytes().await {
        Ok(data) => Some(data),
        Err(_) => {
            result.errors.push(WkdFetchError::NoDataReturned);
            None
        }
    };
    result.data = data;
    result
}

#[cfg(test)]
mod tests {
    use super::super::uri::UserHash;
    use super::*;
    use mockito::ServerGuard;
    use std::fmt::{Display, Formatter};

    #[derive(Debug)]
    pub struct TestUri(String);

    impl Uri<TestUri> for TestUri {
        const PATH: &str = "/.well-known/openpgpkey/hu/iy9q119eutrkn8s1mk4r39qejnbu3n5q?l=Joe.Doe";

        fn new(_domain_part: &str, _local_part: &str, _user_hash: &UserHash) -> Self {
            unimplemented!()
        }
    }

    impl TestUri {
        pub async fn create_test_uri_mock() -> (ServerGuard, TestUri, String, String) {
            let mock_server = mockito::Server::new_async().await;
            let test_path =
                "/.well-known/openpgpkey/hu/iy9q119eutrkn8s1mk4r39qejnbu3n5q?l=Joe.Doe".to_string();
            let test_uri = format!("http://{}{}", mock_server.host_with_port(), test_path);
            let test_policy_path = "/.well-known/openpgpkey/policy".to_string();
            let test_uri = TestUri(test_uri);

            (mock_server, test_uri, test_path, test_policy_path)
        }
    }

    impl Display for TestUri {
        fn fmt(&self, f: &mut Formatter) -> std::result::Result<(), std::fmt::Error> {
            write!(f, "{}", self.0)
        }
    }

    #[tokio::test]
    async fn test_trim_uri() {
        let url = "https://example.org/.well-known/openpgpkey/hu/iy9q119eutrkn8s1mk4r39qejnbu3n5q?l=Joe.Doe";
        let trimmed = trim_uri(url);
        assert_eq!(trimmed, "https://example.org/.well-known/openpgpkey/hu/");
    }

    #[tokio::test]
    async fn test_get_policy_url() {
        let plain_url = "https://example.org/.well-known/openpgpkey/hu/iy9q119eutrkn8s1mk4r39qejnbu3n5q?l=Joe.Doe";
        let domain_url = "https://openpgpkey.example.org/.well-known/openpgpkey/example.org/hu/iy9q119eutrkn8s1mk4r39qejnbu3n5q?l=Joe.Doe";

        let policy_url = get_policy_url(plain_url);
        assert_eq!(
            policy_url,
            Some("https://example.org/.well-known/openpgpkey/policy".to_string())
        );

        let policy_url = get_policy_url(domain_url);
        assert_eq!(
            policy_url,
            Some(
                "https://openpgpkey.example.org/.well-known/openpgpkey/example.org/policy"
                    .to_string()
            )
        );
    }

    #[tokio::test]
    async fn fetch_uri_success() {
        let (mut mock_server, test_uri, test_path, test_policy_path) =
            TestUri::create_test_uri_mock().await;
        mock_server
            .mock("GET", test_path.as_str())
            .with_status(200)
            .with_header("content-type", "application/octet-stream")
            .with_header("access-control-allow-origin", "*")
            .with_body([])
            .create();

        mock_server
            .mock("GET", test_policy_path.as_str())
            .with_status(200)
            // .with_body([])
            .create();

        mock_server
            .mock("HEAD", test_path.as_str())
            .with_status(200)
            .with_header("content-type", "application/octet-stream")
            .with_header("access-control-allow-origin", "*")
            .create();

        let result = fetch_uri(&test_uri).await;
        assert_eq!(result.errors.len(), 0);
        assert!(result.data.is_some());
        mock_server.reset();
    }

    #[tokio::test]
    async fn fetch_uri_invalid_url() {
        let result = fetch_uri(&TestUri("not_a_url".to_string())).await;
        eprintln!("{result:?}");
        assert_eq!(result.errors.len(), 1);
        assert!(matches!(
            result.errors[0],
            WkdFetchError::WkdUriNotValidUrl(_)
        ));
    }

    #[tokio::test]
    async fn fetch_uri_fetch_error() {
        let result = fetch_uri(&TestUri("http://doesnotexist".to_string())).await;
        eprintln!("{result:?}");
        assert_eq!(result.errors.len(), 3);
        assert!(matches!(result.errors[0], WkdFetchError::FailedHeadMethod));
        assert!(matches!(
            result.errors[1],
            WkdFetchError::WkdPolicyFilePathGenerationFailed
        ));
        assert!(matches!(
            result.errors[2],
            WkdFetchError::FailedToFetchUrl(_)
        ));
    }

    #[tokio::test]
    async fn fetch_uri_status_not_200() {
        let (mut mock_server, test_uri, test_path, _test_policy_path) =
            TestUri::create_test_uri_mock().await;

        let mock = mock_server
            .mock("GET", test_path.as_str())
            .with_status(404)
            .create();

        let result = fetch_uri(&test_uri).await;
        eprintln!("{result:?}");
        assert_eq!(result.errors.len(), 3);
        assert!(matches!(result.errors[0], WkdFetchError::FailedHeadMethod));
        assert!(matches!(
            result.errors[1],
            WkdFetchError::WkdPolicyFileNotFound
        ));
        assert!(matches!(result.errors[2], WkdFetchError::StatusNot200(404)));
        mock.assert();
    }

    #[tokio::test]
    async fn fetch_uri_ssl_fail() {
        let (mock_server, _test_uri, test_path, _test_policy_path) =
            TestUri::create_test_uri_mock().await;
        let test_uri = TestUri(format!(
            "https://{}{}",
            mock_server.host_with_port(),
            test_path
        ));
        let result = fetch_uri(&test_uri).await;
        eprintln!("{result:?}");

        assert_eq!(result.errors.len(), 3);
        assert!(matches!(result.errors[0], WkdFetchError::FailedHeadMethod));

        assert!(matches!(
            result.errors[1],
            WkdFetchError::WkdPolicyFileNotFound
        ));
        assert!(matches!(
            result.errors[2],
            WkdFetchError::FailedToFetchUrl(_)
        ));
    }

    #[tokio::test]
    async fn fetch_uri_all_warnings() {
        let (mut mock_server, test_uri, test_path, test_policy_path) =
            TestUri::create_test_uri_mock().await;

        mock_server
            .mock("GET", test_path.as_str())
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_header("access-control-allow-origin", "example.org")
            .create();

        mock_server
            .mock("GET", trim_uri(test_path.as_str()))
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_header("access-control-allow-origin", "example.org")
            .create();

        mock_server
            .mock("GET", test_policy_path.as_str())
            .with_status(404)
            .with_body([])
            .create();

        let result = fetch_uri(&test_uri).await;
        eprintln!("{result:?}");

        assert_eq!(result.errors.len(), 5);
        assert!(matches!(result.errors[0], WkdFetchError::FailedHeadMethod));

        assert!(matches!(
            result.errors[1],
            WkdFetchError::WkdPathShouldNotHaveIndex
        ));
        assert!(matches!(
            result.errors[2],
            WkdFetchError::WkdPolicyFileNotFound
        ));
        assert!(matches!(
            result.errors[3],
            WkdFetchError::ContentTypeNotOctetStream
        ));
        assert!(matches!(
            result.errors[4],
            WkdFetchError::AccessControlAllowOriginNotStar
        ));
        assert!(result.data.is_some());
    }
}
