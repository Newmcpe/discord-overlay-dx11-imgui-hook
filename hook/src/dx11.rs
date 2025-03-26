#![allow(unsafe_op_in_unsafe_fn)]
use crate::cleanup_resources;
use egui_demo_lib::DemoWindows;
use egui_directx11::DirectX11Renderer;
use egui_win32::InputManager;
use once_cell::sync::{Lazy, OnceCell};
use parking_lot::Mutex;
use std::cell::RefCell;
use std::mem::transmute;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Once, OnceLock};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Dxgi::{DXGI_PRESENT, IDXGISwapChain};
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::UI::WindowsAndMessaging;
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, GWLP_WNDPROC, SetWindowLongPtrW, WNDPROC,
};
use windows::core::{HRESULT, Interface, s};

pub(crate) type DXGISwapChainPresentType = unsafe extern "system" fn(
    this: IDXGISwapChain,
    sync_interval: u32,
    flags: DXGI_PRESENT,
) -> HRESULT;

pub static ORIGINAL_PRESENT: OnceLock<DXGISwapChainPresentType> = OnceLock::new();

static WND_PROC: OnceCell<WNDPROC> = OnceCell::new();
static INPUT: Lazy<Mutex<Option<InputManager>>> = Lazy::new(|| Mutex::new(None));
static SHOW_MENU: AtomicBool = AtomicBool::new(true);

static INIT: Once = Once::new();

thread_local! {
    static RENDERER: RefCell<Option<DirectX11Renderer>> = RefCell::new(None);
}

fn should_block_input(msg: u32) -> bool {
    let menu_visible = SHOW_MENU.load(Ordering::SeqCst);

    menu_visible
        && matches!(
            msg,
            WindowsAndMessaging::WM_MOUSEMOVE
                | WindowsAndMessaging::WM_NCMOUSEMOVE
                | WindowsAndMessaging::WM_NCMOUSELEAVE
                | WindowsAndMessaging::WM_LBUTTONDOWN
                | WindowsAndMessaging::WM_LBUTTONDBLCLK
                | WindowsAndMessaging::WM_RBUTTONDOWN
                | WindowsAndMessaging::WM_RBUTTONDBLCLK
                | WindowsAndMessaging::WM_MBUTTONDOWN
                | WindowsAndMessaging::WM_MBUTTONDBLCLK
                | WindowsAndMessaging::WM_XBUTTONDOWN
                | WindowsAndMessaging::WM_XBUTTONDBLCLK
                | WindowsAndMessaging::WM_LBUTTONUP
                | WindowsAndMessaging::WM_RBUTTONUP
                | WindowsAndMessaging::WM_MBUTTONUP
                | WindowsAndMessaging::WM_XBUTTONUP
                | WindowsAndMessaging::WM_MOUSEWHEEL
                | WindowsAndMessaging::WM_MOUSEHWHEEL
                | WindowsAndMessaging::WM_KEYDOWN
                | WindowsAndMessaging::WM_KEYUP
                | WindowsAndMessaging::WM_SYSKEYDOWN
                | WindowsAndMessaging::WM_SYSKEYUP
                | WindowsAndMessaging::WM_SETFOCUS
                | WindowsAndMessaging::WM_KILLFOCUS
                | WindowsAndMessaging::WM_CHAR
                | WindowsAndMessaging::WM_SETCURSOR
                | WindowsAndMessaging::WM_DEVICECHANGE
        )
}

pub fn is_menu_visible() -> bool {
    SHOW_MENU.load(Ordering::SeqCst)
}

unsafe extern "system" fn wnd_proc_proxy(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    INPUT
        .lock()
        .as_mut()
        .unwrap()
        .process(msg, wparam.0, lparam.0);

    let wndproc = WND_PROC.get().expect("WNDPROC is not initialized");

    if msg == WindowsAndMessaging::WM_KEYDOWN && wparam.0 == 0x2D {
        let enabled = SHOW_MENU
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |x| Some(!x))
            .unwrap();
        println!("menu toggled: {}", enabled);
    }

    if should_block_input(msg) {
        return LRESULT(1);
    }

    CallWindowProcW(*wndproc, hwnd, msg, wparam, lparam)
}

pub unsafe extern "system" fn hk_present(
    swap_chain: IDXGISwapChain,
    sync_interval: u32,
    flags: DXGI_PRESENT,
) -> HRESULT {
    INIT.call_once(|| {
        println!("hello from IDXGISwapChain::Present");
        let dx_renderer: DirectX11Renderer =
            DirectX11Renderer::init_from_swapchain(&swap_chain, egui::Context::default()).unwrap();
        let window = swap_chain.GetDesc().unwrap().OutputWindow;

        let wnd_proc: WNDPROC = unsafe {
            let wnd_proc =
                SetWindowLongPtrW(window, GWLP_WNDPROC, wnd_proc_proxy as usize as isize);
            transmute(wnd_proc)
        };

        WND_PROC.set(wnd_proc).unwrap();

        let input_manager = InputManager::new(window);
        INPUT.lock().replace(input_manager);

        RENDERER.with(|cell| {
            *cell.borrow_mut() = Some(dx_renderer);
        });
    });

    RENDERER.with_borrow_mut(|renderer_opt| {
        if let Some(renderer) = renderer_opt.as_mut() {
            let mut input = INPUT.lock();
            let raw_input = input.as_mut().unwrap().collect_input().unwrap();
            let mut shared_state = 1;

            {
                let enabled = is_menu_visible();

                if enabled {
                    let output_window = swap_chain.GetDesc().unwrap().OutputWindow;

                    renderer
                        .paint(
                            &swap_chain,
                            &mut shared_state,
                            raw_input,
                            move |ctx, state| {
                                let mut demo_windows = DemoWindows::default();
                                demo_windows.ui(ctx);

                                egui::Window::new("management")
                                    .default_pos(egui::Pos2::new(100.0, 100.0))
                                    .movable(true)
                                    .show(ctx, |ui| {
                                        ui.button("meow unload")
                                            .on_hover_ui(|ui| {
                                                ui.label("meow mrrp mrrowwww meow meow nya!!! :3");
                                            })
                                            .clicked()
                                            .then(|| {
                                                cleanup_resources();
                                                SetWindowLongPtrW(
                                                    output_window,
                                                    GWLP_WNDPROC,
                                                    WND_PROC.get_unchecked().unwrap() as isize,
                                                );
                                            });
                                    });
                            },
                        )
                        .expect("successful render");
                }
            }
        }
    });

    ORIGINAL_PRESENT.get().unwrap()(swap_chain, sync_interval, flags)
}

pub unsafe fn get_target_address() -> usize {
    let discord_overlay_module = GetModuleHandleA(s!("DiscordHook64.dll"));
    discord_overlay_module.unwrap().0 as usize + 0x1050E0
}
