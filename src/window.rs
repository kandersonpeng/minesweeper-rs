use std::sync::Once;

use bindings::Windows::Graphics::SizeInt32;
use bindings::Windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, PWSTR, RECT, WPARAM};
use bindings::Windows::Win32::System::LibraryLoader::GetModuleHandleW;
use bindings::Windows::Win32::System::WinRT::ICompositorDesktopInterop;
use bindings::Windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, GetClientRect, LoadCursorW, RegisterClassW, CREATESTRUCTW,
    CW_USEDEFAULT, GWLP_USERDATA, HMENU, IDC_ARROW, WM_NCCREATE, WNDCLASSW,
    WS_EX_NOREDIRECTIONBITMAP, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};
use bindings::Windows::UI::Composition::{Compositor, Desktop::DesktopWindowTarget};
use windows::Interface;

use crate::check::CheckHandle;
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
            let instance = GetModuleHandleW(PWSTR(std::ptr::null_mut())).check_handle()?;
            REGISTER_WINDOW_CLASS.call_once(|| {
                let window_class = WNDCLASSW {
                    hCursor: LoadCursorW(HINSTANCE(0), IDC_ARROW).check_handle().unwrap(),
                    hInstance: instance,
                    lpszClassName: class_name.as_pwstr(),
                    lpfnWndProc: Some(Self::wnd_proc),
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
                WS_EX_NOREDIRECTIONBITMAP,
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
        unsafe { compositor_desktop.CreateDesktopWindowTarget(self.handle, is_topmost) }
    }

    fn get_self_from_handle(window: HWND) -> Option<Box<Self>> {
        unsafe {
            let ptr = GetWindowLongPtrW(window, GWLP_USERDATA.0);
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
                    Box::from_raw(create_struct.lpCreateParams as *mut _);
                window_box.handle = window_handle;
                let window_ptr = Box::into_raw(window_box);
                SetWindowLongPtrW(window_handle, GWLP_USERDATA.0, window_ptr as _);
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
        let _ = GetClientRect(window_handle, &mut rect).ok()?;
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        Ok(SizeInt32 {
            Width: width,
            Height: height,
        })
    }
}
