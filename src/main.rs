use std::sync::mpsc::{self, Sender};

use guit_netdog::{
    WlanClient,
    WlanEvent::{self, Exit},
    ssid_to_string,
};
use windows::{
    Win32::NetworkManagement::WiFi::{
        L2_NOTIFICATION_DATA, WLAN_CONNECTION_NOTIFICATION_DATA, WLAN_NOTIFICATION_SOURCE_ACM,
        WLAN_REASON_CODE_SUCCESS, wlan_notification_acm_connection_complete,
    },
    core::Result,
};
use winrt_toast_reborn::{Toast, ToastManager};

unsafe extern "system" fn wlan_callback(
    data: *mut L2_NOTIFICATION_DATA,
    context: *mut std::ffi::c_void,
) {
    if data.is_null() {
        return;
    }

    if context.is_null() {
        return;
    }

    let _ = std::panic::catch_unwind(|| unsafe {
        let sender = &*(context as *const Sender<WlanEvent>);
        let data = &*data;

        if data.NotificationSource == WLAN_NOTIFICATION_SOURCE_ACM
            && data.NotificationCode == wlan_notification_acm_connection_complete.0 as u32
        {
            if data.pData.is_null()
                || (data.dwDataSize as usize)
                    < std::mem::size_of::<WLAN_CONNECTION_NOTIFICATION_DATA>()
            {
                return;
            }

            let data = *data.pData.cast::<WLAN_CONNECTION_NOTIFICATION_DATA>();

            let _ = sender.send(WlanEvent::ConnectionComplete {
                ssid: ssid_to_string(&data.dot11Ssid),
                success: data.wlanReasonCode == WLAN_REASON_CODE_SUCCESS,
            });
        }
    });
}

fn main() -> Result<()> {
    let wlan = WlanClient::open()?;
    let toast_manager = ToastManager::new(ToastManager::POWERSHELL_AUM_ID);

    println!("Negotiated WLAN API version: {}", wlan.negotiated_version());

    let infos = wlan.enum_interfaces()?;

    for info in infos {
        println!("GUID: {:?}", info.guid);
        println!("Description: {}", info.description);
        println!("Connected: {}", info.connected);

        let ssid = wlan
            .query_interface_connection_attributes(info.guid)?
            .wlanAssociationAttributes
            .dot11Ssid;

        println!("ssid: {:?}", ssid_to_string(&ssid));
    }

    let (tx, rx) = mpsc::channel::<WlanEvent>();

    let context = Box::new(tx);

    let context_ptr =
        (context.as_ref() as *const mpsc::Sender<WlanEvent>) as *const std::ffi::c_void;

    let _registration = wlan.register_notification(Some(wlan_callback), Some(context_ptr))?;

    println!("Listening for WLAN notifications...\nPress Enter to exit.");

    let input_sender = context.as_ref().clone();

    let mut toast = Toast::new();
    toast
        .text1("Guit Netdog")
        .text2("Netdog are listening...")
        .text3(
            winrt_toast_reborn::Text::new("Via SMS")
                .with_placement(winrt_toast_reborn::content::text::TextPlacement::Attribution),
        );

    let _ = toast_manager.show(&toast);

    let input_thread = std::thread::spawn(move || {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        let _ = input_sender.send(Exit);
    });

    while let Ok(event) = rx.recv() {
        match event {
            WlanEvent::ConnectionComplete { ssid, success } => {
                if success {
                    println!("Success connect to ssid {:?}.", ssid)
                } else {
                    println!("Fail to connect ssid {:?}.", ssid)
                }
            }
            WlanEvent::Exit => {
                println!("Exiting...");
                break;
            }
        }
    }

    let _ = input_thread.join();

    Ok(())
}
