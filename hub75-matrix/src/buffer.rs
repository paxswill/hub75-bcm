use core::iter;
use core::marker::PhantomData;

use crate::{const_check, const_not_zero};

use super::config::MatrixConfig;

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

    buffer_index: Option<usize>,

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
            buffer_index: None,
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
                    .map(move |plane| (line_index, plane))
            })
            .enumerate()
            .flat_map(|(plane_index, (scanline_index, plane))| {
                plane
                    .buffer
                    .iter_mut()
                    .map(move |word| (plane_index, scanline_index, word))
            })
            .enumerate()
            .map(
                |(word_index, (plane_index, scanline_index, word))| PixelRef {
                    scanline: scanline_index,
                    column: word_index,
                    color_plane: plane_index,
                    word,
                },
            )
    }

    /// Set the address, output enable, and latch values across all pixels in a framebuffer.
    pub(crate) fn set_control_bits(&mut self, latch_blanking_count: u8) {
        let last_column = Self::WORDS_PER_PLANE - 1;
        for pixel_ref in self.iter_mut_pixels() {
            // The first color plane has the previous scanline's address values as we're clocking
            // in the new scanline of data (except for the first row).
            let address_bits = if pixel_ref.color_plane == 0 && pixel_ref.scanline != 0 {
                pixel_ref.scanline - 1
            } else {
                pixel_ref.scanline
            } as u16;
            // Set LAT at the last pixel in each scanline
            let latch = match pixel_ref.column {
                n if n == last_column => 1,
                _ => 0,
            } as u16;
            let blanking_count = latch_blanking_count as usize;
            let oe = if pixel_ref.column < blanking_count
                || (pixel_ref.column > last_column - blanking_count)
            {
                1
            } else {
                0
            } as u16;
            // Mask for RGB bits, then combine everything else
            *pixel_ref.word &= 0x003F & (address_bits << 8) & (oe << 7) & (latch << 6);
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
