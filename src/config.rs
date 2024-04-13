use crate::util::Sealed;

use crate::const_not_zero;

/// `PER_FRAME_DENOMINATOR` is the portion of the panel written to at once.
///
/// Typically RGB matrix panels will be referred to as either a fraction (1/8, 1/16) or a
/// ration (1:8, 1:16). This refers to how many lines are being drawn to as a single scanline
/// For example, if you have a 32 pixel high 1/8 (or 1:8) panel, 4 rows (32 / 8) will be drawn
/// to at a time. If you have a 32 pixel high 1/16 (or 1:16) panel, 2 rows (32 / 16) will
/// be drawn to for each scanline
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MatrixConfig<
    const WIDTH: usize,
    const HEIGHT: usize,
    const CHAIN_LENGTH: usize,
    const COLOR_DEPTH: usize,
    const PER_FRAME_DENOMINATOR: u8,
> {
    /// The number of clock cycles to disable output after changing the latch signal.
    ///
    /// The default value is 2, and there's a maximum value of 4.
    latch_blanking_count: u8,
}

impl<
        const WIDTH: usize,
        const HEIGHT: usize,
        const CHAIN_LENGTH: usize,
        const COLOR_DEPTH: usize,
        const PER_FRAME_DENOMINATOR: u8,
    > Default for MatrixConfig<WIDTH, HEIGHT, CHAIN_LENGTH, COLOR_DEPTH, PER_FRAME_DENOMINATOR>
{
    fn default() -> Self {
        Self {
            latch_blanking_count: Self::DEFAULT_LATCH_BLANKING_COUNT,
        }
    }
}

impl<
        const WIDTH: usize,
        const HEIGHT: usize,
        const CHAIN_LENGTH: usize,
        const COLOR_DEPTH: usize,
        const PER_FRAME_DENOMINATOR: u8,
    > MatrixConfig<WIDTH, HEIGHT, CHAIN_LENGTH, COLOR_DEPTH, PER_FRAME_DENOMINATOR>
{
    const DEFAULT_LATCH_BLANKING_COUNT: u8 = 2;

    const LATCH_BLANKING_COUNT_MAX: u8 = 4;

    const_not_zero!(WIDTH, usize);
    const_not_zero!(HEIGHT, usize);
    const_not_zero!(CHAIN_LENGTH, usize);
    const_not_zero!(COLOR_DEPTH, usize);
    const_not_zero!(PER_FRAME_DENOMINATOR, u8);

    /*
    const WIDTH: usize = const_check!(WIDTH, WIDTH > 0, "WIDTH cannot be 0");

    const HEIGHT: usize = const_check!(HEIGHT, HEIGHT > 0, "HEIGHT cannot be 0");

    const CHAIN_LENGTH: usize =
        const_check!(CHAIN_LENGTH, CHAIN_LENGTH > 0, "CHAIN_LENGTH cannot be 0");

    const COLOR_DEPTH: usize = const_check!(COLOR_DEPTH, COLOR_DEPTH > 0, "COLOR_DEPTH cannot be 0");

    const PER_FRAME_DENOMINATOR: u8 = const_check!(
        PER_FRAME_DENOMINATOR,
        PER_FRAME_DENOMINATOR > 0,
        "PER_FRAME_DENOMINATOR cannot be 0"
    );
    */

    const WORDS_PER_SCANLINE: usize = {
        let pixels_per_row = Self::WIDTH * Self::CHAIN_LENGTH;
        let rows_per_scanline = Self::HEIGHT / (Self::PER_FRAME_DENOMINATOR as usize);
        // Each bit of color depth needs a separate word of storage as we're using BCD
        let pixels_per_scanline = pixels_per_row * (Self::COLOR_DEPTH as usize) * rows_per_scanline;
        // Each word already encodes 2 pixels
        pixels_per_scanline / 2
    };

    const SCANLINES_PER_FRAME: usize = {
        let rows_per_scanline = Self::HEIGHT / (Self::PER_FRAME_DENOMINATOR as usize);
        Self::HEIGHT / rows_per_scanline
    };

    const WORDS_PER_FRAME: usize = { Self::WORDS_PER_SCANLINE * Self::SCANLINES_PER_FRAME };

    pub fn new(latch_blanking_count: u8) -> Self {
        Self {
            latch_blanking_count,
        }
    }

    pub fn latch_blanking_count(&self) -> u8 {
        self.latch_blanking_count
    }

    pub fn set_latch_blanking_count(&mut self, latch_blanking_count: u8) {
        self.latch_blanking_count = latch_blanking_count;
    }

    pub(crate) const fn words_per_scanline(&self) -> usize {
        Self::WORDS_PER_SCANLINE
    }

    pub(crate) const fn scanlines_per_frame(&self) -> usize {
        Self::SCANLINES_PER_FRAME
    }

    pub(crate) const fn words_per_frame(&self) -> usize {
        Self::WORDS_PER_FRAME
    }
}
