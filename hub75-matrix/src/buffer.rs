use core::iter;
use core::marker::PhantomData;

use crate::{const_check, const_not_zero};

use super::config::MatrixConfig;
use super::matrix_word::{MatrixPixel, MatrixWordMut};

pub trait ColorStorage<const COLOR_DEPTH: usize> {
    const COLOR_DEPTH: usize = COLOR_DEPTH;
    fn iter_bits(&self) -> impl Iterator<Item = bool>;
}

macro_rules! impl_color_storage {
    ($type:ty, $depth:literal) => {
        impl ColorStorage<$depth> for $type {
            fn iter_bits(&self) -> impl Iterator<Item = bool> {
                let self_copy = *self;
                (0..$depth).map(move |shift| (self_copy & (1 << shift)) > 0)
            }
        }
    };
}

impl_color_storage!(u8, 1);
impl_color_storage!(u8, 2);
impl_color_storage!(u8, 3);
impl_color_storage!(u8, 4);
impl_color_storage!(u8, 5);
impl_color_storage!(u8, 6);
impl_color_storage!(u8, 7);
impl_color_storage!(u8, 8);

impl_color_storage!(u16, 1);
impl_color_storage!(u16, 2);
impl_color_storage!(u16, 3);
impl_color_storage!(u16, 4);
impl_color_storage!(u16, 5);
impl_color_storage!(u16, 6);
impl_color_storage!(u16, 7);
impl_color_storage!(u16, 8);
impl_color_storage!(u16, 9);
impl_color_storage!(u16, 10);
impl_color_storage!(u16, 11);
impl_color_storage!(u16, 12);
impl_color_storage!(u16, 13);
impl_color_storage!(u16, 14);
impl_color_storage!(u16, 15);
impl_color_storage!(u16, 16);

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct PixelRef<'a> {
    pub(crate) scanline: usize,
    pub(crate) column: usize,
    pub(crate) color_plane: usize,
    pub(crate) word: &'a mut u16,
}

// Defining this here to make it easier if this needs to be added as a parameter later.
pub(crate) const PIXELS_PER_CLOCK: usize = 2;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ColorPlane<
    const WIDTH: usize,
    const HEIGHT: usize,
    const CHAIN_LENGTH: usize,
    const COLOR_DEPTH: usize,
    const PER_FRAME_DENOMINATOR: u8,
    const WORDS_PER_PLANE: usize,
> {
    // We will always need a u16 for the buffer; the RGB bits will take up 6 bits, then OE and LAT
    // bring it up to 8. Any address bits will push it over 8, and there must be at least 1 of them.
    buffer: [u16; WORDS_PER_PLANE],

    _config:
        PhantomData<MatrixConfig<WIDTH, HEIGHT, CHAIN_LENGTH, COLOR_DEPTH, PER_FRAME_DENOMINATOR>>,
}

impl<
        const WIDTH: usize,
        const HEIGHT: usize,
        const CHAIN_LENGTH: usize,
        const COLOR_DEPTH: usize,
        const PER_FRAME_DENOMINATOR: u8,
        const WORDS_PER_PLANE: usize,
    > ColorPlane<WIDTH, HEIGHT, CHAIN_LENGTH, COLOR_DEPTH, PER_FRAME_DENOMINATOR, WORDS_PER_PLANE>
{
    const_not_zero!(WIDTH, usize);
    const_not_zero!(HEIGHT, usize);
    const_not_zero!(CHAIN_LENGTH, usize);
    const_not_zero!(COLOR_DEPTH, usize);
    const_not_zero!(PER_FRAME_DENOMINATOR, u8);

    const WORDS_PER_PLANE: usize = const_check!(
        WORDS_PER_PLANE,
        WORDS_PER_PLANE
            == (WIDTH * CHAIN_LENGTH * HEIGHT
                / (PER_FRAME_DENOMINATOR as usize)
                / PIXELS_PER_CLOCK),
        "WORDS_PER_PLANE must equal WIDTH * CHAIN_LENGTH * HEIGHT / PER_FRAME_DENOMINATOR / 2"
    );

    pub(crate) const fn new() -> Self {
        // Force the compiler to evaluate all the const checks
        let _ = Self::WIDTH;
        let _ = Self::HEIGHT;
        let _ = Self::CHAIN_LENGTH;
        let _ = Self::COLOR_DEPTH;
        let _ = Self::PER_FRAME_DENOMINATOR;
        let _ = Self::WORDS_PER_PLANE;

        Self {
            buffer: [0u16; WORDS_PER_PLANE],
            _config: PhantomData,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Scanline<
    const WIDTH: usize,
    const HEIGHT: usize,
    const CHAIN_LENGTH: usize,
    const COLOR_DEPTH: usize,
    const PER_FRAME_DENOMINATOR: u8,
    const WORDS_PER_PLANE: usize,
> {
    planes: [ColorPlane<
        WIDTH,
        HEIGHT,
        CHAIN_LENGTH,
        COLOR_DEPTH,
        PER_FRAME_DENOMINATOR,
        WORDS_PER_PLANE,
    >; COLOR_DEPTH],

    _config:
        PhantomData<MatrixConfig<WIDTH, HEIGHT, CHAIN_LENGTH, COLOR_DEPTH, PER_FRAME_DENOMINATOR>>,
}

impl<
        const WIDTH: usize,
        const HEIGHT: usize,
        const CHAIN_LENGTH: usize,
        const COLOR_DEPTH: usize,
        const PER_FRAME_DENOMINATOR: u8,
        const WORDS_PER_PLANE: usize,
    > Scanline<WIDTH, HEIGHT, CHAIN_LENGTH, COLOR_DEPTH, PER_FRAME_DENOMINATOR, WORDS_PER_PLANE>
{
    const_not_zero!(WIDTH, usize);
    const_not_zero!(HEIGHT, usize);
    const_not_zero!(CHAIN_LENGTH, usize);
    const_not_zero!(COLOR_DEPTH, usize);
    const_not_zero!(PER_FRAME_DENOMINATOR, u8);

    const WORDS_PER_PLANE: usize = const_check!(
        WORDS_PER_PLANE,
        WORDS_PER_PLANE
            == (WIDTH * CHAIN_LENGTH * HEIGHT
                / (PER_FRAME_DENOMINATOR as usize)
                / PIXELS_PER_CLOCK),
        "WORDS_PER_PLANE must equal WIDTH * CHAIN_LENGTH * HEIGHT / PER_FRAME_DENOMINATOR / 2"
    );

    pub(crate) const fn new() -> Self {
        // Force the compiler to evaluate all the const checks
        let _ = Self::WIDTH;
        let _ = Self::HEIGHT;
        let _ = Self::CHAIN_LENGTH;
        let _ = Self::COLOR_DEPTH;
        let _ = Self::PER_FRAME_DENOMINATOR;
        let _ = Self::WORDS_PER_PLANE;

        let planes = [ColorPlane::<
            WIDTH,
            HEIGHT,
            CHAIN_LENGTH,
            COLOR_DEPTH,
            PER_FRAME_DENOMINATOR,
            WORDS_PER_PLANE,
        >::new(); COLOR_DEPTH];
        Self {
            planes,
            _config: PhantomData,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameBuffer<
    const WIDTH: usize,
    const HEIGHT: usize,
    const CHAIN_LENGTH: usize,
    const COLOR_DEPTH: usize,
    const PER_FRAME_DENOMINATOR: u8,
    const WORDS_PER_PLANE: usize,
    const SCANLINES_PER_FRAME: usize,
> {
    scanlines: [Scanline<
        WIDTH,
        HEIGHT,
        CHAIN_LENGTH,
        COLOR_DEPTH,
        PER_FRAME_DENOMINATOR,
        WORDS_PER_PLANE,
    >; SCANLINES_PER_FRAME],

    configured: bool,

    _config:
        PhantomData<MatrixConfig<WIDTH, HEIGHT, CHAIN_LENGTH, COLOR_DEPTH, PER_FRAME_DENOMINATOR>>,
}

impl<
        const WIDTH: usize,
        const HEIGHT: usize,
        const CHAIN_LENGTH: usize,
        const COLOR_DEPTH: usize,
        const PER_FRAME_DENOMINATOR: u8,
        const WORDS_PER_PLANE: usize,
        const SCANLINES_PER_FRAME: usize,
    >
    FrameBuffer<
        WIDTH,
        HEIGHT,
        CHAIN_LENGTH,
        COLOR_DEPTH,
        PER_FRAME_DENOMINATOR,
        WORDS_PER_PLANE,
        SCANLINES_PER_FRAME,
    >
{
    const_not_zero!(WIDTH, usize);
    const_not_zero!(HEIGHT, usize);
    const_not_zero!(CHAIN_LENGTH, usize);
    const_not_zero!(COLOR_DEPTH, usize);
    const_not_zero!(PER_FRAME_DENOMINATOR, u8);

    pub const WORDS_PER_PLANE: usize = const_check!(
        WORDS_PER_PLANE,
        WORDS_PER_PLANE
            == (WIDTH * CHAIN_LENGTH * HEIGHT
                / (PER_FRAME_DENOMINATOR as usize)
                / PIXELS_PER_CLOCK),
        "WORDS_PER_PLANE must equal WIDTH * CHAIN_LENGTH * HEIGHT / PER_FRAME_DENOMINATOR / 2"
    );

    pub const SCANLINES_PER_FRAME: usize = const_check!(
        SCANLINES_PER_FRAME,
        SCANLINES_PER_FRAME == (HEIGHT / (HEIGHT / PER_FRAME_DENOMINATOR as usize)) && (SCANLINES_PER_FRAME <= 32),
        "SCANLINES_PER_FRAME must equal HEIGHT / (HEIGHT / PER_FRAME_DENOMINATOR), and be less than or equal to 32"
    );

    pub const fn width(&self) -> usize {
        Self::WIDTH
    }

    pub const fn height(&self) -> usize {
        Self::HEIGHT
    }

    pub const fn chain_length(&self) -> usize {
        Self::CHAIN_LENGTH
    }

    pub const fn color_depth(&self) -> usize {
        Self::COLOR_DEPTH
    }

    pub const fn per_frame_denominator(&self) -> u8 {
        Self::PER_FRAME_DENOMINATOR
    }

    pub const fn words_per_plane(&self) -> usize {
        Self::WORDS_PER_PLANE
    }

    pub const fn scanlines_per_frame(&self) -> usize {
        Self::SCANLINES_PER_FRAME
    }

    pub const fn new() -> Self {
        // Force the compiler to evaluate all the const checks
        let _ = Self::WIDTH;
        let _ = Self::HEIGHT;
        let _ = Self::CHAIN_LENGTH;
        let _ = Self::COLOR_DEPTH;
        let _ = Self::PER_FRAME_DENOMINATOR;
        let _ = Self::WORDS_PER_PLANE;
        let _ = Self::SCANLINES_PER_FRAME;

        let scanlines = [Scanline::<
            WIDTH,
            HEIGHT,
            CHAIN_LENGTH,
            COLOR_DEPTH,
            PER_FRAME_DENOMINATOR,
            WORDS_PER_PLANE,
        >::new(); SCANLINES_PER_FRAME];
        Self {
            scanlines,
            configured: false,
            _config: PhantomData,
        }
    }

    pub(crate) fn iter_mut_pixels<'a>(&'a mut self) -> impl Iterator<Item = PixelRef<'a>> {
        self.scanlines
            .iter_mut()
            .enumerate()
            .flat_map(|(line_index, scanline)| {
                scanline
                    .planes
                    .iter_mut()
                    .enumerate()
                    .map(move |(plane_index, plane)| (plane_index, line_index, plane))
            })
            .flat_map(|(plane_index, scanline_index, plane)| {
                plane
                    .buffer
                    .iter_mut()
                    .enumerate()
                    .map(move |(word_index, word)| PixelRef {
                        scanline: scanline_index,
                        column: word_index,
                        color_plane: plane_index,
                        word,
                    })
            })
    }

    pub fn is_configured(&self) -> bool {
        self.configured
    }

    pub(crate) fn configure(&mut self, latch_blanking_count: u8, brightness: u8) {
        // self.set_control_bits(latch_blanking_count);
        // self.set_brightness_bits(latch_blanking_count, brightness);
        self.set_test_pattern();
        self.configured = true;
    }

    /// Set the address, output enable, and latch values across all pixels in a framebuffer.
    pub(crate) fn set_control_bits(&mut self, latch_blanking_count: u8) {
        let last_column = Self::WORDS_PER_PLANE - 1;
        let non_blanked_range_start = latch_blanking_count as usize;
        // Always at least one column, so subtract 1, then subtract the additional blanking
        // columns.
        let non_blanked_range_end = Self::WORDS_PER_PLANE - 1 - latch_blanking_count as usize;
        let non_blanked_range = non_blanked_range_start..non_blanked_range_end;
        for pixel_ref in self.iter_mut_pixels() {
            // The first color plane has the previous scanline's address values as we're clocking
            // in the new scanline of data (except for the first row).
            let address = if pixel_ref.color_plane == 0 {
                (pixel_ref.scanline + Self::SCANLINES_PER_FRAME - 1) % Self::SCANLINES_PER_FRAME
            } else {
                pixel_ref.scanline
            } as u8;
            pixel_ref.word.set_address(address);
            // Set LAT at the last pixel in each scanline
            if pixel_ref.column == last_column {
                pixel_ref.word.set_latch();
            } else {
                pixel_ref.word.clear_latch();
            }
            // Set OE (output enable) for all columns besides the blanking range.
            pixel_ref
                .word
                .set_output_enable_to(!non_blanked_range.contains(&pixel_ref.column));
        }
    }

    pub(crate) fn set_brightness_bits(&mut self, latch_blanking_count: u8, brightness: u8) {
        let last_column = Self::WORDS_PER_PLANE - 1;
        for pixel_ref in self.iter_mut_pixels() {
            let blanking_count = latch_blanking_count as usize;
            if pixel_ref.column >= blanking_count
                || (pixel_ref.column <= last_column - blanking_count)
            {
                // TODO: For now, always enable output. So super bright :/
                *pixel_ref.word |= 1 << 7;
            }
        }
    }

    fn scanline_for(
        &mut self,
        y: usize,
    ) -> &mut Scanline<
        WIDTH,
        HEIGHT,
        CHAIN_LENGTH,
        COLOR_DEPTH,
        PER_FRAME_DENOMINATOR,
        WORDS_PER_PLANE,
    > {
        let offset = y % Self::PER_FRAME_DENOMINATOR as usize;
        &mut self.scanlines[offset]
    }

    pub(crate) fn set_pixel<CS: ColorStorage<COLOR_DEPTH>>(
        &mut self,
        x: usize,
        y: usize,
        red: CS,
        green: CS,
        blue: CS,
    ) {
        let mut bits_index_iter = red
            .iter_bits()
            .zip(green.iter_bits())
            .zip(blue.iter_bits())
            .enumerate();
        let scanline = self.scanline_for(y);
        for (plane_index, ((red, green), blue)) in bits_index_iter {
            let mut color_bits = (red as u8) << 2 | (green as u8) << 1 | blue as u8;
            if y > Self::HEIGHT / 2 {
                color_bits <<= 3;
            }
            let plane = &mut scanline.planes[plane_index];
            plane.buffer[x] &= color_bits as u16;
        }
    }

    pub(crate) fn buffer_iter<'a>(&'a self) -> impl Iterator<Item = &'a [u16]> {
        // Loop from 0 to COLOR_DEPTH
        (0..Self::COLOR_DEPTH)
            // Repeat each color plane index 2^(plane index) times
            .flat_map(|plane| iter::repeat(plane).take(1 << plane))
            // For each color plane, iterate through each scanline index
            .flat_map(|plane| (0..SCANLINES_PER_FRAME).zip(iter::repeat(plane)))
            // Yield a slice for the given scanline index and color plane index
            .map(|(scanline, plane)| &self.scanlines[scanline].planes[plane].buffer[..])
    }

    pub(crate) fn buffer_ptr_iter<'a>(&'a self) -> impl Iterator<Item = (*const u8, usize)> + 'a {
        self.buffer_iter().map(|buf| {
            let ptr_range = buf.as_ptr_range();
            // Safety:
            // From the documentation from byte_offset_from():
            // This is purely a convenience for casting to a u8 pointer and using offset_from
            // on it. See that method for documentation and safety requirements.
            //
            // From offset_from():
            // The primary motivation of this method is for computing the len of an array/slice
            // of T that you are currently representing as a “start” and “end” pointer (and
            // “end” is “one past the end” of the array). In that case, end.offset_from(start)
            // gets you the length of the array.
            // All of the following safety requirements are trivially satisfied for this
            // usecase. [ed: skipping the full list of safety requirements]
            //
            // From slice.as_ptr_range():
            // The returned range is half-open, which means that the end pointer points one
            // past the last element of the slice. This way, an empty slice is represented by
            // two equal pointers, and the difference between the two pointers represents the
            // size of the slice.
            //
            // Together, these statements show that this usage of byte_offset_from() is safe
            let len = unsafe { ptr_range.end.byte_offset_from(ptr_range.start) } as usize;
            (ptr_range.start as _, len)
        })
    }
}

#[macro_export]
macro_rules! alias_frame_buffer {
    ($name:ident, $width:literal, $height:literal, $color_depth:literal, $chain_length:literal, $per_frame_denominator:literal) => {
        type $name = FrameBuffer<
            $width,
            $height,
            $chain_length,
            $color_depth,
            $per_frame_denominator,
            // NOTE: the "2" here is the value of PIXELS_PER_CLOCK
            { $width * $chain_length * $height / $per_frame_denominator / 2 },
            { $height / ($height / $per_frame_denominator) },
        >;
    };
    ($name:ident, $width:literal, $height:literal, $color_depth:literal, $chain_length:literal) => {
        alias_frame_buffer!($name, $width, $height, $color_depth, $chain_length, 16);
    };
    ($name:ident, $width:literal, $height:literal, $color_depth:literal) => {
        alias_frame_buffer!($name, $width, $height, $color_depth, 1);
    };
    ($name:ident, $width:literal, $height:literal) => {
        alias_frame_buffer!($name, $width, $height, 8);
    };
}

#[macro_export]
macro_rules! declare_frame_buffer {
    ($width:literal, $height:literal, $color_depth:literal, $chain_length:literal, $per_frame_denominator:literal) => {{
        FrameBuffer::<
            $width,
            $height,
            $chain_length,
            $color_depth,
            $per_frame_denominator,
            // NOTE: the "2" here is the value of PIXELS_PER_CLOCK
            { $width * $chain_length * $height / $per_frame_denominator / 2 },
            { $height / ($height / $per_frame_denominator) },
        >::new()
    }};
    ($width:literal, $height:literal, $color_depth:literal, $chain_length:literal) => {
        declare_frame_buffer!($width, $height, $color_depth, $chain_length, 16)
    };
    ($width:literal, $height:literal, $color_depth:literal) => {
        declare_frame_buffer!($width, $height, $color_depth, 1)
    };
    ($width:literal, $height:literal) => {
        declare_frame_buffer!($width, $height, 8)
    };
}

#[cfg(test)]
mod test {

    use super::*;

    // Test cases are using std
    extern crate std;
    use std::vec::Vec;

    #[test]
    fn color_plane_length_eighth() {
        let fb = declare_frame_buffer!(64, 32, 8, 1, 8);
        assert_eq!(fb.scanlines[0].planes[0].buffer.len(), 64 * 2);
    }

    #[test]
    fn color_plane_length_eighth_chained() {
        let fb = declare_frame_buffer!(64, 32, 8, 2, 8);
        assert_eq!(fb.scanlines[0].planes[0].buffer.len(), 64 * 2 * 2);
    }

    #[test]
    fn color_plane_length_sixteenth() {
        let fb = declare_frame_buffer!(64, 32, 8, 1, 16);
        assert_eq!(fb.scanlines[0].planes[0].buffer.len(), 64);
    }

    #[test]
    fn color_plane_length_sixteenth_chained() {
        let fb = declare_frame_buffer!(64, 32, 8, 2, 16);
        assert_eq!(fb.scanlines[0].planes[0].buffer.len(), 64 * 2);
    }

    #[test]
    fn color_plane_new_empty() {
        let plane = ColorPlane::<64, 32, 1, 8, 8, 128>::new();
        for element in plane.buffer {
            assert_eq!(element, 0)
        }
    }

    #[test]
    fn scan_line_plane_count_8() {
        let scanline = Scanline::<64, 32, 1, 8, 8, 128>::new();
        assert_eq!(scanline.planes.len(), 8)
    }

    #[test]
    fn scan_line_plane_count_5() {
        let scanline = Scanline::<64, 32, 1, 5, 8, 128>::new();
        assert_eq!(scanline.planes.len(), 5)
    }

    #[test]
    fn fb_num_elements_chain() {
        let fb = declare_frame_buffer!(64, 32, 8, 2);
        let total_words: usize = fb
            .scanlines
            .iter()
            .flat_map(|s| s.planes.iter())
            .map(|p| p.buffer.len())
            .sum();
        // height * width / 2: There's a word needed for every two pixels
        // * 8: There's 8 color planes
        // * 2: There's 2 panels chained together
        assert_eq!(total_words, 64 * 32 / 2 * 8 * 2);
    }

    #[test]
    fn fb_num_elements_eighth_square() {
        let fb = declare_frame_buffer!(32, 32, 8, 1, 8);
        let total_words: usize = fb
            .scanlines
            .iter()
            .flat_map(|s| s.planes.iter())
            .map(|p| p.buffer.len())
            .sum();
        // 32 * 32 / 2: 1 word for every two pixels
        // * 8: There's 8 color planes
        // * 1: (elided) There's only one panel in the chain
        assert_eq!(total_words, 32 * 32 / 2 * 8);
    }

    #[test]
    fn fb_num_elements_sixteenth_square() {
        let fb = declare_frame_buffer!(32, 32, 8, 1, 16);
        let total_words: usize = fb
            .scanlines
            .iter()
            .flat_map(|s| s.planes.iter())
            .map(|p| p.buffer.len())
            .sum();
        // 32 * 32 / 2: 1 word for every two pixels
        // * 8: There's 8 color planes
        // * 1: (elided) There's only one panel in the chain
        assert_eq!(total_words, 32 * 32 / 2 * 8);
    }

    #[test]
    fn fb_initial_blank() {
        let fb = declare_frame_buffer!(32, 32, 8, 1, 16);
        let every_element = fb
            .scanlines
            .iter()
            .flat_map(|s| s.planes.iter())
            .flat_map(|p| p.buffer.iter())
            // Enumerate so we know which element wasn't zero
            .enumerate();
        for (index, element) in every_element {
            assert_eq!(element, &0, "Element {} was not 0", index)
        }
    }

    fn check_frame_buffer_control_bits<
        const W: usize,
        const H: usize,
        const CL: usize,
        const CD: usize,
        const PFD: u8,
        const WPP: usize,
        const SPF: usize,
    >(
        mut fb: FrameBuffer<W, H, CL, CD, PFD, WPP, SPF>,
        latch_blanking_count: usize,
    ) {
        fb.set_control_bits(latch_blanking_count as u8);
        let expected_scanline_count = H / (H / PFD as usize);
        assert_eq!(
            fb.scanlines.len(),
            expected_scanline_count,
            "Unexpected scanline count"
        );
        for scanline_index in 0..expected_scanline_count {
            let scanline = fb.scanlines[scanline_index];
            let expected_buffer_length = W * H / PFD as usize / PIXELS_PER_CLOCK;
            for plane_index in 0..CD {
                let plane = scanline.planes[plane_index];
                assert_eq!(
                    plane.buffer.len(),
                    expected_buffer_length,
                    "Unexpected buffer length"
                );
                for column in 0..expected_buffer_length {
                    let word = plane.buffer[column];
                    // Ensure that OE (output enable) is set for the last element and any within
                    // the given blanking period.
                    if (0..latch_blanking_count).contains(&column) {
                        assert!(
                            word.output_enable(),
                            "Initial OE is not set for column {} on scanline {}, plane {}",
                            column,
                            scanline_index,
                            plane_index
                        );
                    } else if ((expected_buffer_length - 1 - latch_blanking_count)
                        ..expected_buffer_length)
                        .contains(&column)
                    {
                        assert!(
                            word.output_enable(),
                            "Trailing OE is not set for column {} on scanline {}, plane {}",
                            column,
                            scanline_index,
                            plane_index
                        );
                    } else {
                        assert!(
                            !word.output_enable(),
                            "OE is unexpectedly set for column {} on scanline {}, plane {}",
                            column,
                            scanline_index,
                            plane_index
                        );
                    }
                    // Ensure that LAT (latch) is set only on the last column
                    if column == (expected_buffer_length - 1) {
                        assert!(
                            word.latch(),
                            "LAT is not set for column {} on scanline {}, plane {}",
                            column,
                            scanline_index,
                            plane_index
                        );
                    } else {
                        assert!(
                            !word.latch(),
                            "LAT is unexpectedly set for column {} on scanline {}, plane {}",
                            column,
                            scanline_index,
                            plane_index
                        );
                    }
                    // Each scanlines address should be the scanline index, except for the first
                    // color plane which uses the previous scanline's index
                    if plane_index == 0 {
                        let expected_index = if scanline_index == 0 {
                            expected_scanline_count - 1
                        } else {
                            scanline_index - 1
                        };
                        assert_eq!(
                            word.address(),
                            expected_index as u8,
                            "Unexpected index for first color plane"
                        );
                    } else {
                        assert_eq!(
                            word.address(),
                            scanline_index as u8,
                            "Invalid scanline index"
                        );
                    }
                }
            }
        }
    }

    #[test]
    #[test]
    fn set_control_bits_eighth_square() {
        let mut fb = declare_frame_buffer!(32, 32, 8, 1, 8);
        let latch_blanking_count = 0usize;
        check_frame_buffer_control_bits(fb, 0);
    }

    fn set_control_bits_eighth_square_blanking() {
        let mut fb = declare_frame_buffer!(32, 32, 8, 1, 8);
        check_frame_buffer_control_bits(fb, 2);
    }

    #[test]
    fn set_control_bits_sixteenth_square() {
        let mut fb = declare_frame_buffer!(32, 32, 8, 1, 16);
        let latch_blanking_count = 0usize;
        check_frame_buffer_control_bits(fb, 0);
    }

    #[test]
    fn set_control_bits_sixteenth_square_blanking() {
        let mut fb = declare_frame_buffer!(32, 32, 8, 1, 16);
        let latch_blanking_count = 0usize;
        check_frame_buffer_control_bits(fb, 2);
    }

    #[test]
    fn set_upper_red() {
        let mut fb = declare_frame_buffer!(64, 32, 8, 1, 16);
        // Set the control bits; we need to ensure we don't clobber them.
        fb.set_control_bits(0);
        fb.set_brightness_bits(0, 255);
        // Choosing different x and y values so we know the dimensions are correct.
        let x = 5;
        let y = 9;
        let buffer_idx = 5;
        let scanline_idx = 9;
        // Choosing a nicely spread out value that'll hit each color plane differently.
        let red: u8 = 0b10101100;
        // Capture the initial value
        let mut initial_values = Vec::new();
        for plane_idx in 0..fb.color_depth() {
            initial_values.push(fb.scanlines[scanline_idx].planes[plane_idx].buffer[buffer_idx]);
        }
        fb.set_pixel(x, y, red, 0, 0);
        // expected in reverse
        let expected_bits = [false, false, true, true, false, true, false, true];
        for plane_idx in 0..fb.color_depth() {
            let actual_word = fb.scanlines[scanline_idx].planes[plane_idx].buffer[buffer_idx];
            let control_bits = actual_word & 0xFFC0;
            assert_eq!(
                control_bits, initial_values[plane_idx],
                "control bits were clobbered"
            );
            assert_eq!(actual_word & 0x1 != 0, expected_bits[plane_idx]);
        }
    }

    #[test]
    fn set_upper_green() {
        let mut fb = declare_frame_buffer!(64, 32, 8, 1, 16);
        // Set the control bits; we need to ensure we don't clobber them.
        fb.set_control_bits(0);
        fb.set_brightness_bits(0, 255);
        // Choosing different x and y values so we know the dimensions are correct.
        let x = 5;
        let y = 9;
        let buffer_idx = 5;
        let scanline_idx = 9;
        // Choosing a nicely spread out value that'll hit each color plane differently.
        let green: u8 = 0b10101100;
        // Capture the initial value
        let mut initial_values = Vec::new();
        for plane_idx in 0..fb.color_depth() {
            initial_values.push(fb.scanlines[scanline_idx].planes[plane_idx].buffer[buffer_idx]);
        }
        fb.set_pixel(x, y, 0, green, 0);
        // expected in reverse
        let expected_bits = [false, false, true, true, false, true, false, true];
        for plane_idx in 0..fb.color_depth() {
            let actual_word = fb.scanlines[scanline_idx].planes[plane_idx].buffer[buffer_idx];
            let control_bits = actual_word & 0xFFC0;
            assert_eq!(
                control_bits, initial_values[plane_idx],
                "control bits were clobbered"
            );
            assert_eq!(actual_word & 0x2 != 0, expected_bits[plane_idx]);
        }
    }

    #[test]
    fn set_upper_blue() {
        let mut fb = declare_frame_buffer!(64, 32, 8, 1, 16);
        // Set the control bits; we need to ensure we don't clobber them.
        fb.set_control_bits(0);
        fb.set_brightness_bits(0, 255);
        // Choosing different x and y values so we know the dimensions are correct.
        let x = 5;
        let y = 9;
        let buffer_idx = 5;
        let scanline_idx = 9;
        // Choosing a nicely spread out value that'll hit each color plane differently.
        let blue: u8 = 0b10101100;
        // Capture the initial value
        let mut initial_values = Vec::new();
        for plane_idx in 0..fb.color_depth() {
            initial_values.push(fb.scanlines[scanline_idx].planes[plane_idx].buffer[buffer_idx]);
        }
        fb.set_pixel(x, y, 0, 0, blue);
        // expected in reverse
        let expected_bits = [false, false, true, true, false, true, false, true];
        for plane_idx in 0..fb.color_depth() {
            let actual_word = fb.scanlines[scanline_idx].planes[plane_idx].buffer[buffer_idx];
            let control_bits = actual_word & 0xFFC0;
            assert_eq!(
                control_bits, initial_values[plane_idx],
                "control bits were clobbered"
            );
            assert_eq!(actual_word & 0x4 != 0, expected_bits[plane_idx]);
        }
    }

    #[test]
    fn set_lower_red() {
        let mut fb = declare_frame_buffer!(64, 32, 8, 1, 16);
        // Set the control bits; we need to ensure we don't clobber them.
        fb.set_control_bits(0);
        fb.set_brightness_bits(0, 255);
        // Choosing different x and y values so we know the dimensions are correct.
        let x = 5;
        let y = 20;
        let buffer_idx = 5;
        let scanline_idx = 4;
        // Choosing a nicely spread out value that'll hit each color plane differently.
        let red: u8 = 0b10101100;
        // Capture the initial value
        let mut initial_values = Vec::new();
        for plane_idx in 0..fb.color_depth() {
            initial_values.push(fb.scanlines[scanline_idx].planes[plane_idx].buffer[buffer_idx]);
        }
        fb.set_pixel(x, y, red, 0, 0);
        // expected in reverse
        let expected_bits = [false, false, true, true, false, true, false, true];
        for plane_idx in 0..fb.color_depth() {
            let actual_word = fb.scanlines[scanline_idx].planes[plane_idx].buffer[buffer_idx];
            let control_bits = actual_word & 0xFFC0;
            assert_eq!(
                control_bits, initial_values[plane_idx],
                "control bits were clobbered"
            );
            assert_eq!(actual_word & 0x8 != 0, expected_bits[plane_idx]);
        }
    }

    #[test]
    fn set_lower_green() {
        let mut fb = declare_frame_buffer!(64, 32, 8, 1, 16);
        // Set the control bits; we need to ensure we don't clobber them.
        fb.set_control_bits(0);
        fb.set_brightness_bits(0, 255);
        // Choosing different x and y values so we know the dimensions are correct.
        let x = 5;
        let y = 20;
        let buffer_idx = 5;
        let scanline_idx = 4;
        // Choosing a nicely spread out value that'll hit each color plane differently.
        let green: u8 = 0b10101100;
        // Capture the initial value
        let mut initial_values = Vec::new();
        for plane_idx in 0..fb.color_depth() {
            initial_values.push(fb.scanlines[scanline_idx].planes[plane_idx].buffer[buffer_idx]);
        }
        fb.set_pixel(x, y, 0, green, 0);
        // expected in reverse
        let expected_bits = [false, false, true, true, false, true, false, true];
        for plane_idx in 0..fb.color_depth() {
            let actual_word = fb.scanlines[scanline_idx].planes[plane_idx].buffer[buffer_idx];
            let control_bits = actual_word & 0xFFC0;
            assert_eq!(
                control_bits, initial_values[plane_idx],
                "control bits were clobbered"
            );
            assert_eq!(actual_word & 0x10 != 0, expected_bits[plane_idx]);
        }
    }

    #[test]
    fn set_lower_blue() {
        let mut fb = declare_frame_buffer!(64, 32, 8, 1, 16);
        // Set the control bits; we need to ensure we don't clobber them.
        fb.set_control_bits(0);
        fb.set_brightness_bits(0, 255);
        // Choosing different x and y values so we know the dimensions are correct.
        let x = 5;
        let y = 20;
        let buffer_idx = 5;
        let scanline_idx = 4;
        // Choosing a nicely spread out value that'll hit each color plane differently.
        let blue: u8 = 0b10101100;
        // Capture the initial value
        let mut initial_values = Vec::new();
        for plane_idx in 0..fb.color_depth() {
            initial_values.push(fb.scanlines[scanline_idx].planes[plane_idx].buffer[buffer_idx]);
        }
        fb.set_pixel(x, y, 0, 0, blue);
        // expected in reverse
        let expected_bits = [false, false, true, true, false, true, false, true];
        for plane_idx in 0..fb.color_depth() {
            let actual_word = fb.scanlines[scanline_idx].planes[plane_idx].buffer[buffer_idx];
            let control_bits = actual_word & 0xFFC0;
            assert_eq!(
                control_bits, initial_values[plane_idx],
                "control bits were clobbered"
            );
            assert_eq!(actual_word & 0x20 != 0, expected_bits[plane_idx]);
        }
    }
}
