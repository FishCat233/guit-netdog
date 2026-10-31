use windows::Win32::NetworkManagement::WiFi::DOT11_SSID;
use windows::core::GUID;

/// 一个无线网卡的静态信息。
#[derive(Debug)]
pub struct WlanInterfaceInfo {
    pub guid: GUID,
    pub description: String,
    pub connected: bool,
}

/// 把 `DOT11_SSID` 的原始字节转成字符串。
pub fn ssid_to_string(ssid: &DOT11_SSID) -> String {
    let len = ssid.uSSIDLength as usize;
    let bytes = &ssid.ucSSID[..len];

    String::from_utf8_lossy(bytes).into_owned()
}
