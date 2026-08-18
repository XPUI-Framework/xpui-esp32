//! A heap, somewhere for a panic to go, and a way to stop.
//!
//! Nothing a hosted program would think about, and all of it required before
//! the first line of UI code runs.

/// How much of the chip's SRAM `xpui` gets.
///
/// It has to hold the stack of live screens and the view tree `body()` rebuilds
/// on every frame that carries input. The panel's framebuffer is **not** in
/// here — it is a `const`-sized array in `.bss`, which is why 64 kB is enough
/// on a chip that has 400.
const HEAP_SIZE: usize = 64 * 1024;

/// Hands the allocator its memory. Call once, before anything allocates.
///
/// [`xpui::App::new`] allocates on its first line, so this is the first line
/// of every binary here.
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
