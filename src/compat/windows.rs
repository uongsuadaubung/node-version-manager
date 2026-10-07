#![allow(non_snake_case)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::upper_case_acronyms)]

pub mod Win32 {
    pub mod Foundation {
        #[derive(Copy, Clone)]
        pub struct WPARAM(pub usize);
        #[derive(Copy, Clone)]
        pub struct LPARAM(pub isize);
    }
    pub mod UI {
        pub mod WindowsAndMessaging {
            use super::super::Foundation::{LPARAM, WPARAM};

            pub const HWND_BROADCAST: *mut std::ffi::c_void = 0xffff as *mut std::ffi::c_void;
            pub const WM_SETTINGCHANGE: u32 = 0x001A;
            pub const SMTO_ABORTIFHUNG: u32 = 0x0002;

            mod sys {
                #[link(name = "user32")]
                unsafe extern "system" {
                    pub fn SendMessageTimeoutW(
                        hWnd: *mut std::ffi::c_void,
                        Msg: u32,
                        wParam: usize,
                        lParam: isize,
                        fuFlags: u32,
                        uTimeout: u32,
                        lpdwResult: *mut usize,
                    ) -> isize;
                }
            }

            #[allow(clippy::too_many_arguments)]
            pub unsafe fn SendMessageTimeoutW(
                hwnd: *mut std::ffi::c_void,
                msg: u32,
                wparam: WPARAM,
                lparam: LPARAM,
                flags: u32,
                timeout: u32,
                result: Option<*mut usize>,
            ) -> isize {
                unsafe {
                    sys::SendMessageTimeoutW(
                        hwnd,
                        msg,
                        wparam.0,
                        lparam.0,
                        flags,
                        timeout,
                        result.unwrap_or(std::ptr::null_mut()),
                    )
                }
            }
        }
    }
}
