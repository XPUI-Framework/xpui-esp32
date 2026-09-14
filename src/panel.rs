//! A one-bit framebuffer, and the seam where a real panel driver goes.
//!
//! **There is no driver here, and that is deliberate.** `xpui-rp2040` names
//! `uc8151` and `mipidsi` because those are published crates for its panels;
//! for the X3 and the Sticky there is none, and `docs/hardware.md` says why
//! one is not written here. So this owns the framebuffer and stops there: what is
//! missing is one function, [`Panel::present`]. `cargo build` proving a
//! firmware links is not the same as a lit panel.

use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;

/// The panel's memory, one bit per pixel.
///
/// `const` generic over the byte count, so a binary computes the size from
/// its board and [`Panel::new`] asserts the two agree at boot.
pub struct Panel<const BYTES: usize> {
    pixels: [u8; BYTES],
    width: i32,
    height: i32,
}

impl<const BYTES: usize> Panel<BYTES> {
    /// A blank panel of `width` by `height`.
    ///
    /// # Panics
    /// If `BYTES` is not exactly what that size needs. A framebuffer one row
    /// short is a firmware that draws fine and corrupts whatever follows it.
    pub fn new(width: i32, height: i32) -> Self {
        let needed = (width as usize).div_ceil(8) * height as usize;
        assert!(
            BYTES == needed,
            "the framebuffer is the wrong size for this panel"
        );
        Panel {
            // 0 is paper here: `Palette::INK_IS_ON` means a set bit is ink.
            pixels: [0; BYTES],
            width,
            height,
        }
    }

    fn stride(&self) -> usize {
        (self.width as usize).div_ceil(8)
    }

    /// The bytes a driver would push, for whoever writes one.
    pub fn bytes(&self) -> &[u8] {
        &self.pixels
    }

    /// How much ink is on the panel, reported over the serial port on every
    /// repaint.
    ///
    /// Counted a byte at a time, because this runs per frame.
    pub fn ink_count(&self) -> usize {
        let stride = self.stride();
        let used_bits = self.width as usize % 8;
        (0..self.height as usize)
            .map(|y| {
                let row = &self.pixels[y * stride..(y + 1) * stride];
                let (whole, tail) = match used_bits {
                    // The row ends on a byte boundary; every bit is a pixel.
                    0 => (row, 0u32),
                    // The last byte has padding bits nothing draws. Masked off,
                    // or a panel whose width is not a multiple of eight would
                    // report ink that is not on it.
                    used => {
                        let (whole, last) = row.split_at(row.len() - 1);
                        let mask = !0u8 << (8 - used);
                        (whole, (last[0] & mask).count_ones())
                    }
                };
                whole.iter().map(|byte| byte.count_ones()).sum::<u32>() as usize + tail as usize
            })
            .sum()
    }

    /// **Where the panel driver goes.**
    ///
    /// Both boards drive their glass over SPI — the Sticky's is an
    /// SSD1677-class controller — through command and data writes, a waveform
    /// table, and a wait on a BUSY line; none of that is here. A firmware
    /// fills this in, or replaces `Panel` with a driver crate that is already
    /// a `DrawTarget`: then `Backend` holds the driver and this file goes away.
    pub fn present(&mut self) {
        // Nothing. Every byte is in `bytes()`, waiting for a driver.
    }
}

impl<const BYTES: usize> Dimensions for Panel<BYTES> {
    fn bounding_box(&self) -> Rectangle {
        Rectangle::new(
            Point::zero(),
            Size::new(self.width as u32, self.height as u32),
        )
    }
}

impl<const BYTES: usize> DrawTarget for Panel<BYTES> {
    type Color = BinaryColor;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let stride = self.stride();
        for Pixel(point, colour) in pixels {
            // Silently dropped rather than clamped: a clamp would pile every
            // out-of-bounds pixel onto the edge, which paints a visible line
            // where the bug is invisible.
            if point.x < 0 || point.y < 0 || point.x >= self.width || point.y >= self.height {
                continue;
            }
            let index = point.y as usize * stride + point.x as usize / 8;
            let mask = 0x80u8 >> (point.x % 8);
            match colour {
                BinaryColor::On => self.pixels[index] |= mask,
                BinaryColor::Off => self.pixels[index] &= !mask,
            }
        }
        Ok(())
    }

    fn clear(&mut self, colour: Self::Color) -> Result<(), Self::Error> {
        self.pixels.fill(match colour {
            BinaryColor::On => 0xFF,
            BinaryColor::Off => 0x00,
        });
        Ok(())
    }
}
