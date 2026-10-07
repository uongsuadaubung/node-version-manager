#![allow(clippy::upper_case_acronyms)]

use std::ffi::c_void;

type HKEY = *mut c_void;
type LSTATUS = i32;

pub mod enums {
    pub const HKEY_CURRENT_USER: usize = 0x80000001;
    pub const KEY_READ: u32 = 0x20019;
    pub const KEY_WRITE: u32 = 0x20006;
}

#[link(name = "advapi32")]
unsafe extern "system" {
    fn RegOpenKeyExW(
        hKey: HKEY,
        lpSubKey: *const u16,
        ulOptions: u32,
        samDesired: u32,
        phkResult: *mut HKEY,
    ) -> LSTATUS;

    fn RegQueryValueExW(
        hKey: HKEY,
        lpValueName: *const u16,
        lpReserved: *mut u32,
        lpType: *mut u32,
        lpData: *mut u8,
        lpcbData: *mut u32,
    ) -> LSTATUS;

    fn RegSetValueExW(
        hKey: HKEY,
        lpValueName: *const u16,
        Reserved: u32,
        dwType: u32,
        lpData: *const u8,
        cbData: u32,
    ) -> LSTATUS;

    fn RegCloseKey(hKey: HKEY) -> LSTATUS;
}

pub struct RegKey {
    raw: HKEY,
    owned: bool,
}

impl RegKey {
    pub fn predef(val: usize) -> Self {
        Self {
            raw: val as HKEY,
            owned: false,
        }
    }

    pub fn open_subkey_with_flags(&self, subkey: &str, sam: u32) -> std::io::Result<RegKey> {
        let subkey_w: Vec<u16> = subkey.encode_utf16().chain(std::iter::once(0)).collect();
        let mut result_key: HKEY = std::ptr::null_mut();
        let status = unsafe {
            RegOpenKeyExW(
                self.raw,
                subkey_w.as_ptr(),
                0,
                sam,
                &mut result_key,
            )
        };
        if status == 0 {
            Ok(RegKey {
                raw: result_key,
                owned: true,
            })
        } else {
            Err(std::io::Error::from_raw_os_error(status))
        }
    }

    pub fn get_value(&self, name: &str) -> std::io::Result<String> {
        let name_w: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
        let mut val_type: u32 = 0;
        let mut byte_len: u32 = 0;

        let status = unsafe {
            RegQueryValueExW(
                self.raw,
                name_w.as_ptr(),
                std::ptr::null_mut(),
                &mut val_type,
                std::ptr::null_mut(),
                &mut byte_len,
            )
        };
        if status != 0 {
            return Err(std::io::Error::from_raw_os_error(status));
        }

        let mut buffer: Vec<u8> = vec![0u8; byte_len as usize];
        let status = unsafe {
            RegQueryValueExW(
                self.raw,
                name_w.as_ptr(),
                std::ptr::null_mut(),
                &mut val_type,
                buffer.as_mut_ptr(),
                &mut byte_len,
            )
        };
        if status != 0 {
            return Err(std::io::Error::from_raw_os_error(status));
        }

        let u16_slice: &[u16] = unsafe {
            std::slice::from_raw_parts(buffer.as_ptr() as *const u16, (byte_len / 2) as usize)
        };
        let clean_slice = match u16_slice.split_last() {
            Some((&0, rest)) => rest,
            _ => u16_slice,
        };
        String::from_utf16(clean_slice).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    pub fn set_value(&self, name: &str, val: &str) -> std::io::Result<()> {
        let name_w: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
        let val_w: Vec<u16> = val.encode_utf16().chain(std::iter::once(0)).collect();
        let byte_len = (val_w.len() * 2) as u32;

        let status = unsafe {
            RegSetValueExW(
                self.raw,
                name_w.as_ptr(),
                0,
                1, // REG_SZ
                val_w.as_ptr() as *const u8,
                byte_len,
            )
        };
        if status == 0 {
            Ok(())
        } else {
            Err(std::io::Error::from_raw_os_error(status))
        }
    }
}

impl Drop for RegKey {
    fn drop(&mut self) {
        if self.owned && !self.raw.is_null() {
            unsafe {
                RegCloseKey(self.raw);
            }
        }
    }
}
