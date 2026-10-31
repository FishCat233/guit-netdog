use guit_netdog::{WlanClient, ssid_to_string};
use windows::core::Result;

fn main() -> Result<()> {
    let wlan = WlanClient::open()?;

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

    Ok(())
}
