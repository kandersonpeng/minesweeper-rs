fn main() {
    windows::build!(
        windows::foundation::numerics::{Vector2, Vector3},
        windows::foundation::TimeSpan,
        windows::graphics::SizeInt32,
        windows::system::DispatcherQueueController,
        windows::ui::composition::desktop::DesktopWindowTarget,
        windows::ui::composition::{
            AnimationIterationBehavior, CompositionAnimation, CompositionBatchTypes,
            CompositionBorderMode, CompositionColorBrush, CompositionContainerShape,
            CompositionEllipseGeometry, CompositionGeometry, CompositionNineGridBrush,
            CompositionScopedBatch, CompositionShape, CompositionShapeCollection,
            CompositionSpriteShape, Compositor, ContainerVisual, ShapeVisual, SpriteVisual,
            Vector3KeyFrameAnimation, VisualCollection,
        },
        windows::ui::{Color, Colors},
        windows::win32::display_devices::RECT,
        windows::win32::system_services::{
            CreateDispatcherQueueController, BOOL, PWSTR, WM_MOUSEMOVE, WM_SIZE, WM_SIZING,
            GetModuleHandleW, CW_USEDEFAULT, HINSTANCE, IDC_ARROW, LRESULT, WM_LBUTTONDOWN,
            WM_DESTROY, WS_OVERLAPPEDWINDOW, WS_VISIBLE, WS_EX_NOREDIRECTIONBITMAP, E_HANDLE,
            WM_RBUTTONDOWN, WM_NCCREATE, GWLP_USERDATA, E_FAIL,
        },
        windows::win32::windows_and_messaging::HWND,
        windows::win32::winrt::ICompositorDesktopInterop,
        windows::win32::menus_and_resources::{LoadCursorW, HMENU, WNDPROC, HCURSOR},
        windows::win32::windows_and_messaging::{
            CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
            RegisterClassW, HWND, LPARAM, MSG, WNDCLASSW, WPARAM, TranslateMessage, CREATESTRUCTW,
            GetClientRect,
        },
    );
}
