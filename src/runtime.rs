//! A heap, and a way to stop.
//!
//! Nothing a hosted program would think about, and the heap is required before
//! the first line of UI code runs. The panic handler is not here: it is
//! `esp-backtrace`'s, which each binary links with `use esp_backtrace as _;`.

/// How much of the chip's SRAM `xpui` gets.
///
/// It holds the leaked backend — and with it the panel's framebuffer: 52,272
/// bytes on the X3, four fifths of this, and 48,000 on the Sticky — the stack
/// of live screens, and the view tree `body()` rebuilds on every frame that
/// carries input. Raise it before adding a screen that buffers anything.
const HEAP_SIZE: usize = 64 * 1024;

/// Hands the allocator its memory.
///
/// Call it once, before anything allocates. [`xpui::App::new`] allocates on
/// its first line, so this is the first line of every binary here.
pub fn init_heap() {
    esp_alloc::heap_allocator!(size: HEAP_SIZE);
}

/// Stops, keeping whatever the panel last showed.
///
/// E-ink holds its image with the power off, so the most useful thing a
/// firmware with nowhere to go can do is leave the last frame up.
pub fn park() -> ! {
    loop {
        // Nothing to wait for. A real firmware would sleep the core here.
        core::hint::spin_loop();
    }
}
