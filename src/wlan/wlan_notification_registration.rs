use windows::Win32::{
    Foundation::ERROR_SUCCESS,
    NetworkManagement::WiFi::{WLAN_NOTIFICATION_SOURCE_NONE, WlanRegisterNotification},
};

use crate::WlanClient;

#[derive(Debug)]
pub struct WlanNotificationRegistration<'a> {
    pub client: &'a WlanClient,
}

impl Drop for WlanNotificationRegistration<'_> {
    fn drop(&mut self) {
        unsafe {
            let result = WlanRegisterNotification(
                self.client.handle(),
                WLAN_NOTIFICATION_SOURCE_NONE,
                false,
                None,
                None,
                None,
                None,
            );

            if result != ERROR_SUCCESS.0 {
                eprintln!("Failed to unregister WLAN notification: {}", result)
            }
        }
    }
}
