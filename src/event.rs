#[derive(Debug)]
pub enum WlanEvent {
    ConnectionComplete { ssid: String, success: bool },
    Exit,
}
