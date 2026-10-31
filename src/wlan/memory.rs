use windows::Win32::NetworkManagement::WiFi::WlanFreeMemory;

/// WLAN API 返回的堆内存的 RAII 包装：出作用域自动 `WlanFreeMemory`。
pub struct WlanMemory<T> {
    ptr: *mut T,
}

impl<T> Default for WlanMemory<T> {
    fn default() -> Self {
        Self {
            ptr: std::ptr::null_mut(),
        }
    }
}

impl<T> WlanMemory<T> {
    pub fn new(ptr: *mut T) -> Self {
        Self { ptr }
    }

    pub fn as_ptr(&self) -> *mut T {
        self.ptr
    }

    pub fn as_mut_ptr(&mut self) -> *mut *mut T {
        &mut self.ptr
    }
}

impl<T> Drop for WlanMemory<T> {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                WlanFreeMemory(self.ptr as _);
            }
        }
    }
}
