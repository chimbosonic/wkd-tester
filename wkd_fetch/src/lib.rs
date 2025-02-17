use bytes::Bytes;
use reqwest::Url;
use wkd_uri::Uri;

use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
pub enum WkdFetchError {
    #[error("WKD URI provided is not a valid URL")]
    #[diagnostic(code(wkd_fetch))]
    WkdUriNotValidUrl(#[from] url::ParseError),

    #[error("WKD URI provided is not a valid URL")]
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
    StatusNot200,
}

#[derive(Debug)]
pub struct WkdFetch {
    pub errors: Vec<WkdFetchError>,
    pub data: Option<Bytes>,
}

pub async fn fetch_uri<T>(
    uri: &(impl Uri<T> + std::fmt::Debug + std::string::ToString),
) -> Result<WkdFetch, WkdFetchError> {
    let url = match Url::parse(&uri.to_string()) {
        Ok(url) => url,
        Err(err) => {
            return Err(WkdFetchError::WkdUriNotValidUrl(err));
        }
    };

    let response = match reqwest::get(url).await {
        Ok(response) => response,
        Err(err) => {
            return Err(WkdFetchError::FailedToFetchUrl(err));
        }
    };

    let mut result = WkdFetch {
        errors: Vec::new(),
        data: None,
    };

    if response.status().as_u16() != 200 {
        return Err(WkdFetchError::StatusNot200);
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
    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::fmt::{Display, Formatter};

    use super::*;
    use mockito::ServerGuard;
    use wkd_uri::{Uri, UserHash};

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

        let mock = mock_server
            .mock("GET", test_path.as_str())
            .with_status(200)
            .with_header("content-type", "application/octet-stream")
            .with_header("access-control-allow-origin", "*")
            .with_body([])
            .create();

        let result = fetch_uri(&test_uri).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.errors.len(), 0);
        assert!(result.data.is_some());

        mock.assert();
    }

    #[tokio::test]
    async fn fetch_uri_invalid_url() {
        let result = fetch_uri(&TestUri("not_a_url".to_string())).await;
        assert!(result.is_err());
        let result = result.unwrap_err();
        assert!(matches!(result, WkdFetchError::WkdUriNotValidUrl(_)));
    }

    #[tokio::test]
    async fn fetch_uri_fetch_error() {
        let result = fetch_uri(&TestUri("http://doesnotexist".to_string())).await;
        assert!(result.is_err());
        let result = result.unwrap_err();
        assert!(matches!(result, WkdFetchError::FailedToFetchUrl(_)));
    }

    #[tokio::test]
    async fn fetch_uri_status_not_200() {
        let (mut mock_server, test_uri, test_path) = TestUri::create_test_uri_mock().await;

        let mock = mock_server
            .mock("GET", test_path.as_str())
            .with_status(404)
            .create();

        let result = fetch_uri(&test_uri).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), WkdFetchError::StatusNot200));
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
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
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

        let result = fetch_uri(&test_uri).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.errors.len(), 2);
        assert!(matches!(
            result.errors[0],
            WkdFetchError::ContentTypeNotOctetStream
        ));
        assert!(matches!(
            result.errors[1],
            WkdFetchError::AccessControlAllowOriginNotStar
        ));
        assert!(result.data.is_some());

        mock.assert();
    }
}
