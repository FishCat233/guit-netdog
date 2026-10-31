use std::os::raw::c_void;

use windows::Win32::Foundation::{E_FAIL, ERROR_SUCCESS, HANDLE};
use windows::Win32::NetworkManagement::WiFi::{
    WLAN_CONNECTION_ATTRIBUTES, WLAN_INTERFACE_INFO_LIST, WLAN_NOTIFICATION_CALLBACK,
    WLAN_NOTIFICATION_SOURCE_ALL, WLAN_OPCODE_VALUE_TYPE, WlanCloseHandle, WlanEnumInterfaces,
    WlanOpenHandle, WlanQueryInterface, WlanRegisterNotification, wlan_interface_state_connected,
    wlan_intf_opcode_current_connection,
};
use windows::core::{Error, GUID, Result};

use crate::error::win32_result_to_error;
use crate::wlan::WlanNotificationRegistration;
use crate::wlan::memory::WlanMemory;
use crate::wlan::types::WlanInterfaceInfo;

/// WLAN API 句柄的 RAII 包装
#[derive(Debug)]
pub struct WlanClient {
    handle: HANDLE,
    negotiated_version: u32,
}

impl WlanClient {
    /// 打开 WLAN 句柄（版本 2）。
    pub fn open() -> Result<Self> {
        unsafe {
            let mut negotiated_version = 0;
            let mut handle = HANDLE::default();

            let result = WlanOpenHandle(2, None, &mut negotiated_version, &mut handle);

            if result != ERROR_SUCCESS.0 {
                return Err(win32_result_to_error(result));
            }

            Ok(Self {
                handle,
                negotiated_version,
            })
        }
    }

    pub fn handle(&self) -> HANDLE {
        self.handle
    }

    /// 与服务端协商出的 WLAN API 版本。
    pub fn negotiated_version(&self) -> u32 {
        self.negotiated_version
    }

    /// 枚举当前系统中的无线网络接口。
    pub fn enum_interfaces(&self) -> Result<Vec<WlanInterfaceInfo>> {
        unsafe {
            let mut interface_list_memory: WlanMemory<WLAN_INTERFACE_INFO_LIST> =
                WlanMemory::default();

            let result =
                WlanEnumInterfaces(self.handle(), None, interface_list_memory.as_mut_ptr());

            if result != ERROR_SUCCESS.0 {
                return Err(win32_result_to_error(result));
            }

            if interface_list_memory.as_ptr().is_null() {
                // i dont know what situation happend can go here.
                return Ok(Vec::new());
            }

            let list = *interface_list_memory.as_ptr();

            let mut infos = Vec::with_capacity(list.dwNumberOfItems as usize);

            for i in 0..list.dwNumberOfItems as usize {
                let interface_info = &list.InterfaceInfo[i];

                let description = &interface_info.strInterfaceDescription;
                let description_len = description
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(description.len());

                infos.push(WlanInterfaceInfo {
                    guid: interface_info.InterfaceGuid,
                    description: String::from_utf16_lossy(&description[..description_len]),
                    connected: interface_info.isState == wlan_interface_state_connected,
                });
            }

            Ok(infos)
        }
    }

    /// 查询指定接口当前的连接属性。
    pub fn query_interface_connection_attributes(
        &self,
        guid: GUID,
    ) -> Result<WLAN_CONNECTION_ATTRIBUTES> {
        unsafe {
            let mut data: WlanMemory<std::ffi::c_void> = WlanMemory::default();
            let mut data_size: u32 = 0;
            let mut data_type = WLAN_OPCODE_VALUE_TYPE::default();

            let result = WlanQueryInterface(
                self.handle(),
                &guid,
                wlan_intf_opcode_current_connection,
                None,
                &mut data_size,
                data.as_mut_ptr(),
                Some(&mut data_type),
            );

            if result != ERROR_SUCCESS.0 {
                return Err(win32_result_to_error(result));
            }

            if data.as_ptr().is_null()
                || (data_size as usize) < std::mem::size_of::<WLAN_CONNECTION_ATTRIBUTES>()
            {
                return Err(Error::from_hresult(E_FAIL));
            }

            Ok(data.as_ptr().cast::<WLAN_CONNECTION_ATTRIBUTES>().read())
        }
    }

    pub fn register_notification(
        &self,
        callback: WLAN_NOTIFICATION_CALLBACK,
        context: Option<*const c_void>,
    ) -> Result<WlanNotificationRegistration<'_>> {
        unsafe {
            let result = WlanRegisterNotification(
                self.handle(),
                WLAN_NOTIFICATION_SOURCE_ALL,
                false, // 不忽略重复消息
                callback,
                context,
                None,
                None,
            );

            if result != ERROR_SUCCESS.0 {
                return Err(win32_result_to_error(result));
            }
        }

        Ok(WlanNotificationRegistration { client: self })
    }
}

impl Drop for WlanClient {
    fn drop(&mut self) {
        unsafe {
            WlanCloseHandle(self.handle, None);
        }
    }
}
