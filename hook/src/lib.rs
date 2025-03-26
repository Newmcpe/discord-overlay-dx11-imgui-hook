#![allow(unsafe_op_in_unsafe_fn)]
mod dx11;

use crate::dx11::ORIGINAL_PRESENT;
use minhook::MinHook;
use once_cell::sync::OnceCell;
use std::ffi::c_void;
use std::panic::set_hook;
use windows::Win32::System::Console::{AllocConsole, FreeConsole};
use windows::Win32::System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};
use windows::Win32::System::Threading::Sleep;
use windows::core::BOOL;
use windows::core::imp::{FreeLibrary, HMODULE};

//yes i know this is a bad way to do it but i'm lazy
#[derive(Debug)]
struct ThreadSafeHMODULE(HMODULE);
unsafe impl Send for ThreadSafeHMODULE {}
unsafe impl Sync for ThreadSafeHMODULE {}
static HMODULE: OnceCell<Option<ThreadSafeHMODULE>> = OnceCell::new();

pub unsafe fn cleanup_resources() {
    unsafe {
        if let Some(Some(hmodule)) = HMODULE.get() {
            println!("we can unload!");
            FreeLibrary(hmodule.0);
        }
        Sleep(3000);
        FreeConsole().unwrap();
        MinHook::disable_all_hooks().unwrap();
    }
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn DllMain(
    _hInstDll: HMODULE,
    fdwReason: u32,
    _lpvReserved: *mut c_void,
) -> BOOL {
    match fdwReason {
        DLL_PROCESS_ATTACH => {
            AllocConsole().unwrap();
            set_hook(Box::new(|panic_info| {
                println!("panic: {:?}", panic_info);
                loop {}
            }));

            let target_addr = dx11::get_target_address() as *mut *mut c_void;
            println!("Target address: {:p}", target_addr);

            let result = MinHook::create_hook(*target_addr as _, dx11::hk_present as _);
            println!("MinHook::create_hook result: {:?}", result);
            if result.is_ok() {
                ORIGINAL_PRESENT.get_or_init(|| std::mem::transmute(result.unwrap()));
            }

            let result = MinHook::enable_all_hooks();
            println!("MinHook::enable_all_hooks result: {:?}", result);

            if HMODULE.get().is_none() {
                HMODULE.set(Some(ThreadSafeHMODULE(_hInstDll))).unwrap();
            }
        }
        DLL_PROCESS_DETACH => {
            println!("DLL detached meeow");
        }
        _ => {}
    }

    BOOL(1)
}
