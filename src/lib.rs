#![feature(restricted_std)]

mod log;
mod offsets;

use offsets::{LOC_RUN_SCENE_UPDATE, LOC_VSYNC_COUNT_THREAD};
use skyline::hooks::InlineCtx;
use std::sync::atomic::{AtomicBool, Ordering};

// IBinder::~IBinder sits at nnSdk base + 0x240, so subtracting it yields the module base
unsafe fn nnsdk_base() -> u64 {
    let mut symbol = 0usize;
    skyline::nn::ro::LookupSymbol(&mut symbol, b"_ZN7android7IBinderD1Ev\0".as_ptr());
    (symbol - 0x240) as u64
}

static mut SET_INTERVAL_CALL: u64 = 0;
static mut SET_INTERVAL_STORE: u64 = 0;

#[skyline::hook(replace = SET_INTERVAL_CALL)]
unsafe fn set_interval_call(window: u64, _: i32) {
    call_original!(window, 0);
}

#[skyline::hook(replace = SET_INTERVAL_STORE, inline)]
unsafe fn set_interval_store(ctx: &mut InlineCtx) {
    ctx.registers[8].set_x(0);
}

static VSYNC_TICK: AtomicBool = AtomicBool::new(false);

#[skyline::hook(offset = LOC_VSYNC_COUNT_THREAD.get_offset_in_memory().unwrap(), inline)]
unsafe fn vsync_count_thread(_: &InlineCtx) {
    VSYNC_TICK.store(true, Ordering::SeqCst);
}

// With vsync off the scene loop free-runs, so gate each update on the vsync tick
#[skyline::hook(offset = LOC_RUN_SCENE_UPDATE.get_offset_in_memory().unwrap(), inline)]
unsafe fn run_scene_update(_: &InlineCtx) {
    let mut dummy = [0u8; 0x100];
    while !VSYNC_TICK.swap(false, Ordering::SeqCst) {
        skyline::nn::hid::GetNpadFullKeyState(dummy.as_mut_ptr() as _, &0);
    }
}

#[skyline::main(name = "ssbu-vsync-disabler")]
pub fn main() {
    log::startup();
    unsafe {
        let base = nnsdk_base();
        SET_INTERVAL_CALL = base + 0x429d60;
        SET_INTERVAL_STORE = base + 0x26e94;

        if ensure_hooks!(LOC_VSYNC_COUNT_THREAD, LOC_RUN_SCENE_UPDATE) {
            skyline::install_hooks!(
                set_interval_call,
                set_interval_store,
                run_scene_update,
                vsync_count_thread,
            );
        }
    }
}
