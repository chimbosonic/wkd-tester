use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct StaticContentConfig {
    /// URL to your website used in the footer
    pub footer_hoster_url: &'static str,
    /// Your name or organization name used in the footer
    pub footer_hoster_name: &'static str,
    /// The base URL ie: protocol://host
    pub base_url: &'static str,
    /// This sets the path used in static content. Path must start with a slash but not end with a slash. For `/` use `""`.
    pub root_path: &'static str,
}

pub struct ServerConfig {
    /// The host address to bind to.
    pub host: &'static str,
    /// The port to bind to.
    pub port: u16,
}

/// Make sure to update this with your information if you are self hosting.
pub static STATIC_CONTENT_CONFIG: StaticContentConfig = StaticContentConfig {
    footer_hoster_url: "https://chimbosonic.com",
    footer_hoster_name: "Alexis Lowe",
    base_url: "https://wkd.dp42.dev",
    root_path: "/wkd",
};

/// Make sure to update this with your information if you are self hosting.
pub static SERVER_CONFIG: ServerConfig = ServerConfig {
    host: "0.0.0.0",
    port: 7070,
};

#[cfg(feature = "embed-static")]
pub static INDEX_HBS: &str = include_str!("../static/index.hbs");

#[cfg(feature = "embed-static")]
pub static SITEMAP_HBS: &str = include_str!("../static/sitemap.hbs");
