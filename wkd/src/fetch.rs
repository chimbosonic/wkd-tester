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

    #[error("Content-Type header is not set to 'application/octet-stream'. This may cause issues with parsing")]
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

    match client.head(url.clone()).send().await {
        Ok(response) => {
            if response.status().as_u16() != 200 {
                result.errors.push(WkdFetchError::FailedHeadMethod);
            }
        }
        Err(_err) => {
            result.errors.push(WkdFetchError::FailedHeadMethod);
        }
    };

    let index_url = trim_uri(url.as_str());
    if let Ok(response) = client.get(index_url).send().await {
        if response.status().as_u16() == 200 {
            result.errors.push(WkdFetchError::WkdPathShouldNotHaveIndex);
        }
    };

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

    if let Some(header_value) = response.headers().get("content-type") {
        if header_value != "application/octet-stream" {
            result.errors.push(WkdFetchError::ContentTypeNotOctetStream);
        }
    }

    if let Some(header_value) = response.headers().get("access-control-allow-origin") {
        if header_value != "*" {
            result
                .errors
                .push(WkdFetchError::AccessControlAllowOriginNotStar);
        }
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
        const PATH: &str = "/test";

        fn new(_domain_part: &str, _local_part: &str, _user_hash: &UserHash) -> Self {
            unimplemented!()
        }
    }

    impl TestUri {
        pub async fn create_test_uri_mock() -> (ServerGuard, TestUri, String) {
            let mock_server = mockito::Server::new_async().await;
            let test_uri = format!("http://{}/test", mock_server.host_with_port());
            let test_uri = TestUri(test_uri);

            return (mock_server, test_uri, "/test".to_string());
        }
    }

    impl Display for TestUri {
        fn fmt(&self, f: &mut Formatter) -> std::result::Result<(), std::fmt::Error> {
            write!(f, "{}", self.0)
        }
    }

    #[tokio::test]
    async fn fetch_uri_success() {
        let (mut mock_server, test_uri, test_path) = TestUri::create_test_uri_mock().await;

        let mock_get = mock_server
            .mock("GET", test_path.as_str())
            .with_status(200)
            .with_header("content-type", "application/octet-stream")
            .with_header("access-control-allow-origin", "*")
            .with_body([])
            .create();

        let mock_head = mock_server
            .mock("HEAD", test_path.as_str())
            .with_status(200)
            .with_header("content-type", "application/octet-stream")
            .with_header("access-control-allow-origin", "*")
            .create();

        let result = fetch_uri(&test_uri).await;
        assert_eq!(result.errors.len(), 0);
        assert!(result.data.is_some());

        mock_get.assert();
        mock_head.assert();
    }

    #[tokio::test]
    async fn fetch_uri_invalid_url() {
        let result = fetch_uri(&TestUri("not_a_url".to_string())).await;
        eprintln!("{:?}", result);
        assert_eq!(result.errors.len(), 1);
        assert!(matches!(
            result.errors[0],
            WkdFetchError::WkdUriNotValidUrl(_)
        ));
    }

    #[tokio::test]
    async fn fetch_uri_fetch_error() {
        let result = fetch_uri(&TestUri("http://doesnotexist".to_string())).await;
        eprintln!("{:?}", result);
        assert_eq!(result.errors.len(), 2);
        assert!(matches!(result.errors[0], WkdFetchError::FailedHeadMethod));
        assert!(matches!(
            result.errors[1],
            WkdFetchError::FailedToFetchUrl(_)
        ));
    }

    #[tokio::test]
    async fn fetch_uri_status_not_200() {
        let (mut mock_server, test_uri, test_path) = TestUri::create_test_uri_mock().await;

        let mock = mock_server
            .mock("GET", test_path.as_str())
            .with_status(404)
            .create();

        let result = fetch_uri(&test_uri).await;
        eprintln!("{:?}", result);
        assert_eq!(result.errors.len(), 2);
        assert!(matches!(result.errors[0], WkdFetchError::FailedHeadMethod));
        assert!(matches!(result.errors[1], WkdFetchError::StatusNot200(404)));
        mock.assert();
    }

    #[tokio::test]
    async fn fetch_uri_ssl_fail() {
        let (mock_server, _test_uri, test_path) = TestUri::create_test_uri_mock().await;
        let test_uri = TestUri(format!(
            "https://{}{}",
            mock_server.host_with_port(),
            test_path
        ));
        let result = fetch_uri(&test_uri).await;
        eprintln!("{:?}", result);

        assert_eq!(result.errors.len(), 2);
        assert!(matches!(result.errors[0], WkdFetchError::FailedHeadMethod));
        assert!(matches!(
            result.errors[1],
            WkdFetchError::FailedToFetchUrl(_)
        ));
    }

    #[tokio::test]
    async fn fetch_uri_all_warnings() {
        let (mut mock_server, test_uri, test_path) = TestUri::create_test_uri_mock().await;

        let mock = mock_server
            .mock("GET", test_path.as_str())
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_header("access-control-allow-origin", "example.org")
            .create();

        let mock_index = mock_server
            .mock("GET", trim_uri(test_path.as_str()))
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_header("access-control-allow-origin", "example.org")
            .create();

        let result = fetch_uri(&test_uri).await;
        eprintln!("{:?}", result);

        assert_eq!(result.errors.len(), 4);
        assert!(matches!(result.errors[0], WkdFetchError::FailedHeadMethod));

        assert!(matches!(
            result.errors[1],
            WkdFetchError::WkdPathShouldNotHaveIndex
        ));
        assert!(matches!(
            result.errors[2],
            WkdFetchError::ContentTypeNotOctetStream
        ));
        assert!(matches!(
            result.errors[3],
            WkdFetchError::AccessControlAllowOriginNotStar
        ));
        assert!(result.data.is_some());

        mock.assert();
        mock_index.assert();
    }
}
