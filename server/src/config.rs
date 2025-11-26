use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct FooterData {
    pub host_url: &'static str,
    pub host_name: &'static str,
    pub libera_pay_user: &'static str,
}

#[derive(Clone, Serialize)]
pub struct SiteMapData {
    pub base_url: &'static str,
}

pub struct ServerConfig {
    pub host: &'static str,
    pub port: u16,
}

/// Make sure to update this with your information if you are self hosting.
pub static FOOTER_DATA: FooterData = FooterData {
    host_url: "https://chimbosonic.com",
    host_name: "Alexis Lowe",
    libera_pay_user: "chimbosonic",
};

pub static SITEMAP_DATA: SiteMapData = SiteMapData {
    base_url: "https://wkd.dp42.dev",
};

pub static SERVER_CONFIG: ServerConfig = ServerConfig {
    host: "0.0.0.0",
    port: 7070,
};
