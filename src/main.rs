mod check;
mod comp_assets;
mod comp_ui;
mod dispatcher_queue;
mod minesweeper;
mod numerics;
mod visual_grid;
mod wide_string;
mod window;

use dispatcher_queue::create_dispatcher_queue_controller_for_current_thread;
use minesweeper::Minesweeper;
use window::{get_window_size, Window};

use bindings::Windows::{
    Foundation::Numerics::Vector2,
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT},
        UI::WindowsAndMessaging::{
            DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage, TranslateMessage, MSG,
            WM_DESTROY, WM_LBUTTONDOWN, WM_MOUSEMOVE, WM_RBUTTONDOWN, WM_SIZE, WM_SIZING,
        },
    },
    UI::Composition::Compositor,
};

fn run() -> windows::Result<()> {
    windows::initialize_sta()?;
    let _controller = create_dispatcher_queue_controller_for_current_thread()?;

    let window_width = 800;
    let window_height = 600;

    let compositor = Compositor::new()?;
    let root = compositor.CreateContainerVisual()?;
    root.SetRelativeSizeAdjustment(Vector2::new(1.0, 1.0))?;

    let window_size = Vector2 {
        X: window_width as f32,
        Y: window_height as f32,
    };
    let mut game = Minesweeper::new(&root, &window_size)?;

    let message_handler = move |window_handle, message, wparam, lparam| -> LRESULT {
        unsafe {
            match message {
                WM_DESTROY => {
                    PostQuitMessage(0);
                    return LRESULT(0);
                }
                WM_MOUSEMOVE => {
                    let (x, y) = get_mouse_position(lparam);
                    let point = Vector2 {
                        X: x as f32,
                        Y: y as f32,
                    };
                    game.on_pointer_moved(&point).unwrap();
                }
                WM_SIZE | WM_SIZING => {
                    let new_size = get_window_size(window_handle).unwrap();
                    let new_size = Vector2 {
                        X: new_size.Width as f32,
                        Y: new_size.Height as f32,
                    };
                    game.on_parent_size_changed(&new_size).unwrap();
                }
                WM_LBUTTONDOWN => {
                    game.on_pointer_pressed(false, false).unwrap();
                }
                WM_RBUTTONDOWN => {
                    game.on_pointer_pressed(true, false).unwrap();
                }
                _ => {}
            }
            DefWindowProcW(window_handle, message, wparam, lparam)
        }
    };

    let window = Window::new("Minesweeper", window_width, window_height, message_handler)?;

    let target = window.create_window_target(&compositor, false)?;
    target.SetRoot(&root)?;

    let mut message = MSG::default();
    unsafe {
        while GetMessageW(&mut message, HWND(0), 0, 0).into() {
            TranslateMessage(&mut message);
            DispatchMessageW(&mut message);
        }
    }

    windows::HRESULT::from_win32(message.wParam.0 as u32).ok()
}

fn main() {
    let result = run();

    // We do this for nicer HRESULT printing when errors occur.
    if let Err(error) = result {
        error.code().unwrap();
    }
}

fn get_mouse_position(lparam: LPARAM) -> (isize, isize) {
    let x = lparam.0 & 0xffff;
    let y = (lparam.0 >> 16) & 0xffff;
    (x, y)
}
