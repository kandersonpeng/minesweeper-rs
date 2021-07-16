use bindings::Windows::Win32::Foundation::{E_HANDLE, HINSTANCE, HWND};
use bindings::Windows::Win32::UI::WindowsAndMessaging::HCURSOR;

pub trait CheckHandle {
    fn check_handle(&self) -> windows::Result<Self>
    where
        Self: Sized;
}

macro_rules! impl_check_handle {
    ($handle_type:ty) => {
        impl CheckHandle for $handle_type {
            fn check_handle(&self) -> windows::Result<$handle_type> {
                if *self != 0 {
                    Ok(*self)
                } else {
                    windows::HRESULT::from_thread().ok()?;
                    Err(windows::Error::fast_error(windows::HRESULT(E_HANDLE.0)))
                }
            }
        }
    };
}

macro_rules! impl_check_handle_binding {
    ($handle_type:ty) => {
        impl CheckHandle for $handle_type {
            fn check_handle(&self) -> windows::Result<$handle_type> {
                if self.0 != 0 {
                    Ok(*self)
                } else {
                    windows::HRESULT::from_thread().ok()?;
                    Err(windows::Error::fast_error(windows::HRESULT(E_HANDLE.0)))
                }
            }
        }
    };
}

impl_check_handle!(isize);
impl_check_handle!(u16);
impl_check_handle_binding!(HWND);
impl_check_handle_binding!(HCURSOR);
impl_check_handle_binding!(HINSTANCE);
