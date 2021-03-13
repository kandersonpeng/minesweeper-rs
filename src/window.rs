use std::sync::Once;

use bindings::windows::graphics::SizeInt32;
use bindings::windows::ui::composition::{desktop::DesktopWindowTarget, Compositor};
use bindings::windows::win32::display_devices::RECT;
use bindings::windows::win32::menus_and_resources::{LoadCursorW, HMENU};
use bindings::windows::win32::system_services::GWLP_USERDATA;
use bindings::windows::win32::system_services::WM_NCCREATE;
use bindings::windows::win32::system_services::{
    GetModuleHandleW, CW_USEDEFAULT, HINSTANCE, IDC_ARROW, LRESULT, PWSTR,
    WS_EX_NOREDIRECTIONBITMAP, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};
use bindings::windows::win32::windows_and_messaging::DefWindowProcW;
use bindings::windows::win32::windows_and_messaging::{
    CreateWindowExW, RegisterClassW, HWND, LPARAM, WNDCLASSW, WPARAM,
};
use bindings::windows::win32::windows_and_messaging::{GetClientRect, CREATESTRUCTW};
use bindings::windows::win32::winrt::ICompositorDesktopInterop;
use windows::Interface;

use crate::check::{CheckBool, CheckHandle};
use crate::wide_string::ToWide;

static REGISTER_WINDOW_CLASS: Once = Once::new();
static WINDOW_CLASS_NAME: &str = "Minesweeper.Window";

#[link(name = "user32")]
extern "system" {
    fn SetWindowLongPtrW(h_wnd: HWND, n_index: i32, dw_new_long: i64) -> i64;

    fn GetWindowLongPtrW(h_wnd: HWND, n_index: i32) -> i64;
}

#[derive(Debug)]
pub struct Window<F: FnMut(HWND, u32, WPARAM, LPARAM) -> LRESULT> {
    handle: HWND,
    message_handler: Box<F>,
}

impl<F: FnMut(HWND, u32, WPARAM, LPARAM) -> LRESULT> Window<F> {
    pub fn new(
        title: &str,
        width: u32,
        height: u32,
        message_handler: F,
    ) -> windows::Result<Box<Self>> {
        let class_name = WINDOW_CLASS_NAME.to_wide();
        let title = title.to_wide();
        unsafe {
            let instance = HINSTANCE(GetModuleHandleW(PWSTR(std::ptr::null_mut())).check_handle()?);
            REGISTER_WINDOW_CLASS.call_once(|| {
                let window_class = WNDCLASSW {
                    h_cursor: LoadCursorW(HINSTANCE(0), PWSTR(IDC_ARROW as *mut u16))
                        .check_handle()
                        .unwrap(),
                    h_instance: instance,
                    lpsz_class_name: class_name.as_pwstr(),
                    lpfn_wnd_proc: Some(Self::wnd_proc),
                    ..Default::default()
                };
                let _ = RegisterClassW(&window_class).check_handle().unwrap();
            });

            let result = Box::new(Self {
                handle: HWND(0),
                message_handler: Box::new(message_handler),
            });
            let result_ptr = Box::into_raw(result);
            let _ = CreateWindowExW(
                WS_EX_NOREDIRECTIONBITMAP as u32,
                class_name.as_pwstr(),
                title.as_pwstr(),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                width as i32,
                height as i32,
                HWND(0),
                HMENU(0),
                instance,
                result_ptr as *mut _,
            )
            .check_handle()?;
            Ok(Box::from_raw(result_ptr))
        }
    }

    pub fn create_window_target(
        &self,
        compositor: &Compositor,
        is_topmost: bool,
    ) -> windows::Result<DesktopWindowTarget> {
        let compositor_desktop: ICompositorDesktopInterop = compositor.cast()?;
        let mut result = None;
        unsafe {
            compositor_desktop
                .CreateDesktopWindowTarget(self.handle, is_topmost, &mut result)
                .and_some(result)
        }
    }

    fn get_self_from_handle(window: HWND) -> Option<Box<Self>> {
        unsafe {
            let ptr = GetWindowLongPtrW(window, GWLP_USERDATA);
            if ptr != 0 {
                Some(Box::from_raw(ptr as *mut _))
            } else {
                None
            }
        }
    }

    extern "system" fn wnd_proc(
        window_handle: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        unsafe {
            if message == WM_NCCREATE as u32 {
                let create_struct = lparam.0 as *mut CREATESTRUCTW;
                let create_struct = create_struct.as_mut().unwrap();
                let mut window_box: Box<Self> =
                    Box::from_raw(create_struct.lp_create_params as *mut _);
                window_box.handle = window_handle;
                let window_ptr = Box::into_raw(window_box);
                SetWindowLongPtrW(window_handle, GWLP_USERDATA, window_ptr as _);
            } else if let Some(this) = Self::get_self_from_handle(window_handle) {
                let this = Box::into_raw(this);
                let this = this.as_mut().unwrap();
                return (this.message_handler)(window_handle, message, wparam, lparam);
            }

            DefWindowProcW(window_handle, message, wparam, lparam)
        }
    }
}

pub fn get_window_size(window_handle: HWND) -> windows::Result<SizeInt32> {
    unsafe {
        let mut rect = RECT::default();
        let _ = GetClientRect(window_handle, &mut rect).check_bool()?;
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        Ok(SizeInt32 { width, height })
    }
}
