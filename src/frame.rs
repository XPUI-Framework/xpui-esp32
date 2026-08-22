//! The frame loop, written once for both boards.
//!
//! The same shape as the RP2040's and the simulator's — begin a frame, tick,
//! and paint only if the framework asked — without embassy, because nothing
//! here awaits: there is no driver to wait on and no input to poll.
//!
//! When a panel driver arrives, this is where it changes:
//! [`Panel::present`](crate::Panel::present) becomes a real flush, and if that
//! flush suspends, `Backend::loan_display` is what lets it — see
//! `examples/rp2040/src/frame.rs::run_async`.

use esp_hal::delay::Delay;
use esp_hal::time::Instant;
use gallery::wire;
use xpui::App;
use xpui::screen::Screen;
use xpui_boards::Board;
use xpui_eg::{Backend, Palette};

use crate::Panel;
use crate::runtime::park;

/// How often the loop wakes.
///
/// The same interval `examples/rp2040` uses, and it exists for the same
/// reason: **a loop with no wait in it spins the core at full clock for the
/// life of the battery.** There is no input source on these boards yet, so
/// without this the firmware would do nothing, forever, at full power.
const FRAME_INTERVAL_MS: u32 = 10;

/// Installs `panel` as the host, runs `root` on it, and reports what it drew.
///
/// The report is over the serial port and it matters more than it looks:
/// without a panel driver, an ink count is the only evidence the frame reached
/// pixels rather than stopping somewhere in layout.
pub fn run<const BYTES: usize, S>(panel: Panel<BYTES>, board: Board, root: S) -> !
where
    S: Screen + 'static,
{
    // Sized from the board, so a screen developed in the simulator window and
    // the same screen here lay out against identical numbers.
    let backend = wire(panel, board, Palette::INK_IS_ON).leaked();
    // Safety: one panel, one thread, and nothing has rendered yet.
    unsafe { xpui::host::install(backend) };

    let mut app = App::new(root)
        // Nothing owns a screen under this one, as on any device: the root
        // declines to finish rather than the loop dropping Back at the pin.
        .keep_root();

    // Painted once, or an e-ink panel holds whatever survived reset — which is
    // not a blank screen, it is the previous firmware's last frame.
    app.render();
    backend.clear_dirty();
    present(backend, board);

    let delay = Delay::new();
    while app.is_running() {
        // A real clock, not a constant. `Clock::millis` is a host contract and
        // key auto-repeat is timed against it; answering 0 forever would be a
        // fixed lie rather than a missing feature. Wrapping at 49 days is the
        // framework's own contract — what reads it measures short intervals.
        backend.begin_frame(Instant::now().duration_since_epoch().as_millis() as u32);

        app.tick();
        if app.render_if_dirty() {
            backend.clear_dirty();
            present(backend, board);
        }

        delay.delay_millis(FRAME_INTERVAL_MS);
    }

    // The root screen finished. A device has nowhere to return to.
    park()
}

/// Pushes the frame, and says what was on it.
///
/// The ink count goes out over the serial port on **every** repaint rather
/// than once at boot: with no panel driver it is the only evidence a frame
/// reached pixels rather than stopping somewhere in layout, and one reading
/// cannot tell a live loop from a frozen one.
fn present<const BYTES: usize>(backend: &Backend<Panel<BYTES>>, board: Board) {
    let ink = backend.with_display(|panel| {
        panel.present();
        panel.ink_count()
    });
    esp_println::println!(
        "xpui: {} {}x{}, {} ink pixels",
        board.name,
        board.width,
        board.height,
        ink
    );
}
