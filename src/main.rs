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

use bindings::windows::{
    foundation::numerics::Vector2,
    ui::composition::Compositor,
    win32::{
        system_services::{
            LRESULT, WM_DESTROY, WM_LBUTTONDOWN, WM_MOUSEMOVE, WM_RBUTTONDOWN, WM_SIZE, WM_SIZING,
        },
        windows_and_messaging::{
            DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage, TranslateMessage, HWND,
            LPARAM, MSG,
        },
    },
};

fn run() -> windows::Result<()> {
    windows::initialize_sta()?;
    let _controller = create_dispatcher_queue_controller_for_current_thread()?;

    let window_width = 800;
    let window_height = 600;

    let compositor = Compositor::new()?;
    let root = compositor.create_container_visual()?;
    root.set_relative_size_adjustment(Vector2 { x: 1.0, y: 1.0 })?;

    let window_size = Vector2 {
        x: window_width as f32,
        y: window_height as f32,
    };
    let mut game = Minesweeper::new(&root, &window_size)?;

    let message_handler = move |window_handle, message, wparam, lparam| -> LRESULT {
        unsafe {
            match message as i32 {
                WM_DESTROY => {
                    PostQuitMessage(0);
                    return LRESULT(0);
                }
                WM_MOUSEMOVE => {
                    let (x, y) = get_mouse_position(lparam);
                    let point = Vector2 {
                        x: x as f32,
                        y: y as f32,
                    };
                    game.on_pointer_moved(&point).unwrap();
                }
                WM_SIZE | WM_SIZING => {
                    let new_size = get_window_size(window_handle).unwrap();
                    let new_size = Vector2 {
                        x: new_size.width as f32,
                        y: new_size.height as f32,
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
    target.set_root(&root)?;

    let mut message = MSG::default();
    unsafe {
        while GetMessageW(&mut message, HWND(0), 0, 0).into() {
            TranslateMessage(&mut message);
            DispatchMessageW(&mut message);
        }
    }

    windows::ErrorCode::from_win32(message.w_param.0 as u32).ok()

    /*
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                let size = Vector2 {
                    x: size.width as f32,
                    y: size.height as f32,
                };
                game.on_parent_size_changed(&size).unwrap();
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                let point = Vector2 {
                    x: position.x as f32,
                    y: position.y as f32,
                };
                game.on_pointer_moved(&point).unwrap();
            }
            Event::WindowEvent {
                event: WindowEvent::MouseInput { state, button, .. },
                ..
            } => {
                if state == ElementState::Pressed {
                    game.on_pointer_pressed(button == MouseButton::Right, false)
                        .unwrap();
                }
            }
            _ => (),
        }
    });
    */
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
