use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct FooterData {
    host_url: &'static str,
    host_name: &'static str,
    libera_pay_user: &'static str,
}

/// Make sure to update this with your information if you are self hosting.
pub static FOOTER_DATA: FooterData = FooterData {
    host_url: "https://chimbosonic.com",
    host_name: "Alexis Lowe",
    libera_pay_user: "chimbosonic",
};
