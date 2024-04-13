use core::iter;
use core::ops::{Deref, DerefMut};
use embedded_graphics_core::draw_target::DrawTarget;
use embedded_graphics_core::geometry::{OriginDimensions, Size};
use embedded_graphics_core::pixelcolor::PixelColor;
use embedded_graphics_core::Pixel;

use crate::{const_check, const_not_zero};

use super::buffer::FrameBuffer;
use super::color::Color;
use super::config::MatrixConfig;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MatrixError {
    OutOfBounds,
}

pub struct RgbMatrix<
    'a,
    ColorType,
    const WIDTH: usize,
    const HEIGHT: usize,
    const CHAIN_LENGTH: usize,
    const COLOR_DEPTH: usize,
    const PER_FRAME_DENOMINATOR: u8,
    const WORDS_PER_PLANE: usize,
    const SCANLINES_PER_FRAME: usize,
    const BITMAP_ELEMENTS: usize,
> {
    config: MatrixConfig<WIDTH, HEIGHT, CHAIN_LENGTH, COLOR_DEPTH, PER_FRAME_DENOMINATOR>,

    // Stuck with this multidimensional array because we're using const generics and we can't use
    // them in const expressions.
    pixel_buffer: [[[ColorType; WIDTH]; CHAIN_LENGTH]; HEIGHT],

    dirty_bitmap: [u32; BITMAP_ELEMENTS],

    brightness: u8,

    brightness_dirty: bool,

    pending_frame_buffer: Option<
        &'a mut FrameBuffer<
            WIDTH,
            HEIGHT,
            CHAIN_LENGTH,
            COLOR_DEPTH,
            PER_FRAME_DENOMINATOR,
            WORDS_PER_PLANE,
            SCANLINES_PER_FRAME,
        >,
    >,
}

impl<
        'a,
        ColorType,
        const WIDTH: usize,
        const HEIGHT: usize,
        const CHAIN_LENGTH: usize,
        const COLOR_DEPTH: usize,
        const PER_FRAME_DENOMINATOR: u8,
        const WORDS_PER_PLANE: usize,
        const SCANLINES_PER_FRAME: usize,
        const BITMAP_ELEMENTS: usize,
    >
    RgbMatrix<
        'a,
        ColorType,
        WIDTH,
        HEIGHT,
        CHAIN_LENGTH,
        COLOR_DEPTH,
        PER_FRAME_DENOMINATOR,
        WORDS_PER_PLANE,
        SCANLINES_PER_FRAME,
        BITMAP_ELEMENTS,
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
                / crate::buffer::PIXELS_PER_CLOCK),
        "WORDS_PER_PLANE must equal WIDTH * CHAIN_LENGTH * HEIGHT / PER_FRAME_DENOMINATOR / 2"
    );

    pub const SCANLINES_PER_FRAME: usize = const_check!(
        SCANLINES_PER_FRAME,
        SCANLINES_PER_FRAME == (HEIGHT / (HEIGHT / PER_FRAME_DENOMINATOR as usize)) && (SCANLINES_PER_FRAME <= 32),
        "SCANLINES_PER_FRAME must equal HEIGHT / (HEIGHT / PER_FRAME_DENOMINATOR), and be less than or equal to 32"
    );

    const BITMAP_ELEMENTS: usize = const_check!(
        BITMAP_ELEMENTS,
        BITMAP_ELEMENTS == (HEIGHT * WIDTH * CHAIN_LENGTH / (u32::BITS as usize)),
        "BITMAP_ELEMENTS must be HEIGHT * WIDTH * CHAIN_LENGTH / 32"
    );

    const CHAIN_WIDTH: usize = Self::WIDTH * Self::CHAIN_LENGTH;

    const MAX_WIDTH: usize = Self::CHAIN_WIDTH - 1;

    const MAX_HEIGHT: usize = Self::HEIGHT - 1;

    const DEFAULT_BRIGHTNESS: u8 = 128;

    const fn height(&self) -> usize {
        Self::HEIGHT
    }

    const fn panel_width(&self) -> usize {
        Self::WIDTH
    }

    const fn chain_width(&self) -> usize {
        Self::CHAIN_WIDTH
    }

    pub fn brightness(&self) -> u8 {
        self.brightness
    }

    pub fn set_brightness(&mut self, new_brightness: u8) {
        self.brightness_dirty = new_brightness != self.brightness;
        self.brightness = new_brightness;
    }

    pub fn configure_frame_buffer(
        &self,
        frame_buffer: &mut FrameBuffer<
            WIDTH,
            HEIGHT,
            CHAIN_LENGTH,
            COLOR_DEPTH,
            PER_FRAME_DENOMINATOR,
            WORDS_PER_PLANE,
            SCANLINES_PER_FRAME,
        >,
    ) {
        frame_buffer.configure(self.config.latch_blanking_count(), self.brightness);
    }
}

impl<
        'a,
        ColorType,
        const WIDTH: usize,
        const HEIGHT: usize,
        const CHAIN_LENGTH: usize,
        const COLOR_DEPTH: usize,
        const PER_FRAME_DENOMINATOR: u8,
        const WORDS_PER_PLANE: usize,
        const SCANLINES_PER_FRAME: usize,
        const BITMAP_ELEMENTS: usize,
    >
    RgbMatrix<
        'a,
        ColorType,
        WIDTH,
        HEIGHT,
        CHAIN_LENGTH,
        COLOR_DEPTH,
        PER_FRAME_DENOMINATOR,
        WORDS_PER_PLANE,
        SCANLINES_PER_FRAME,
        BITMAP_ELEMENTS,
    >
where
    ColorType: PartialEq + Color<COLOR_DEPTH>,
{
    pub fn set_pixel(
        &mut self,
        x: usize,
        y: usize,
        new_color: ColorType,
    ) -> Result<(), MatrixError> {
        // Discard early out of bounds coordinates. We also can't use pattern matching here
        // because we're using const generics for all the bounds.
        if x >= Self::CHAIN_WIDTH {
            return Err(MatrixError::OutOfBounds);
        }
        if y >= Self::HEIGHT {
            return Err(MatrixError::OutOfBounds);
        }
        // Calculate which panel in the chain this x coordinate refers to
        let panel_index = x as usize / Self::WIDTH;
        let panel_x = x as usize % Self::WIDTH;
        let y = y as usize;
        // Set the dirty flag before changing the color
        let old_color = &self.pixel_buffer[y][panel_index][panel_x];
        if old_color != &new_color {
            let overall_bit_index = y * Self::CHAIN_WIDTH + x;
            let element_index = overall_bit_index / u32::BITS as usize;
            let bit_index = overall_bit_index % u32::BITS as usize;
            self.dirty_bitmap[element_index] |= 1 << bit_index;
            if let Some(frame_buffer) = &mut self.pending_frame_buffer {
                frame_buffer.set_pixel(x, y, new_color.red(), new_color.green(), new_color.blue());
            }
            self.pixel_buffer[y][panel_index][panel_x] = new_color;
        }
        Ok(())
    }
}

impl<
        'a,
        ColorType,
        const WIDTH: usize,
        const HEIGHT: usize,
        const CHAIN_LENGTH: usize,
        const COLOR_DEPTH: usize,
        const PER_FRAME_DENOMINATOR: u8,
        const WORDS_PER_PLANE: usize,
        const SCANLINES_PER_FRAME: usize,
        const BITMAP_ELEMENTS: usize,
    >
    RgbMatrix<
        'a,
        ColorType,
        WIDTH,
        HEIGHT,
        CHAIN_LENGTH,
        COLOR_DEPTH,
        PER_FRAME_DENOMINATOR,
        WORDS_PER_PLANE,
        SCANLINES_PER_FRAME,
        BITMAP_ELEMENTS,
    >
where
    ColorType: Default + Copy + Color<COLOR_DEPTH>,
{
    pub fn new(
        config: MatrixConfig<WIDTH, HEIGHT, CHAIN_LENGTH, COLOR_DEPTH, PER_FRAME_DENOMINATOR>,
    ) -> Self {
        // Force the compiler to evaluate all the const checks
        let _ = Self::WIDTH;
        let _ = Self::HEIGHT;
        let _ = Self::CHAIN_LENGTH;
        let _ = Self::COLOR_DEPTH;
        let _ = Self::PER_FRAME_DENOMINATOR;
        let _ = Self::WORDS_PER_PLANE;
        let _ = Self::SCANLINES_PER_FRAME;
        let _ = Self::BITMAP_ELEMENTS;

        Self {
            config,
            pixel_buffer: [[[ColorType::default(); WIDTH]; CHAIN_LENGTH]; HEIGHT],
            dirty_bitmap: [0u32; BITMAP_ELEMENTS],
            brightness: Self::DEFAULT_BRIGHTNESS,
            brightness_dirty: false,
            pending_frame_buffer: None,
        }
    }

    pub fn set_pending(
        &mut self,
        mut new_frame_buffer: &'a mut FrameBuffer<
            WIDTH,
            HEIGHT,
            CHAIN_LENGTH,
            COLOR_DEPTH,
            PER_FRAME_DENOMINATOR,
            WORDS_PER_PLANE,
            SCANLINES_PER_FRAME,
        >,
    ) -> Option<
        &'a mut FrameBuffer<
            WIDTH,
            HEIGHT,
            CHAIN_LENGTH,
            COLOR_DEPTH,
            PER_FRAME_DENOMINATOR,
            WORDS_PER_PLANE,
            SCANLINES_PER_FRAME,
        >,
    > {
        self.update_dirty(&mut new_frame_buffer);
        self.pending_frame_buffer.replace(new_frame_buffer)
    }

    fn update_dirty(
        &mut self,
        frame_buffer: &mut FrameBuffer<
            WIDTH,
            HEIGHT,
            CHAIN_LENGTH,
            COLOR_DEPTH,
            PER_FRAME_DENOMINATOR,
            WORDS_PER_PLANE,
            SCANLINES_PER_FRAME,
        >,
    ) {
        if self.brightness_dirty {
            frame_buffer.set_brightness_bits(self.config.latch_blanking_count(), self.brightness);
            self.brightness_dirty = false;
        }
        for (element_index, element) in self
            .dirty_bitmap
            .iter_mut()
            .enumerate()
            .filter(|(_, e)| **e != 0)
        {
            for bit_index in 0..u32::BITS {
                let masked = *element & (1 << bit_index);
                if masked != 0 {
                    let overall_bit_index = (element_index as u32 * u32::BITS + bit_index) as usize;
                    let y = overall_bit_index / Self::CHAIN_WIDTH;
                    let x = overall_bit_index % Self::CHAIN_WIDTH;
                    let panel_index = x / Self::WIDTH;
                    let panel_x = x % Self::WIDTH;
                    let color = self.pixel_buffer[y][panel_index][panel_x];
                    frame_buffer.set_pixel(x, y, color.red(), color.green(), color.blue());
                    *element &= !masked;
                }
            }
        }
    }
}

impl<
        'a,
        ColorType,
        const WIDTH: usize,
        const HEIGHT: usize,
        const CHAIN_LENGTH: usize,
        const COLOR_DEPTH: usize,
        const PER_FRAME_DENOMINATOR: u8,
        const WORDS_PER_PLANE: usize,
        const SCANLINES_PER_FRAME: usize,
        const BITMAP_ELEMENTS: usize,
    > OriginDimensions
    for RgbMatrix<
        'a,
        ColorType,
        WIDTH,
        HEIGHT,
        CHAIN_LENGTH,
        COLOR_DEPTH,
        PER_FRAME_DENOMINATOR,
        WORDS_PER_PLANE,
        SCANLINES_PER_FRAME,
        BITMAP_ELEMENTS,
    >
{
    fn size(&self) -> Size {
        Size {
            width: self.chain_width() as u32,
            height: self.height() as u32,
        }
    }
}

impl<
        'a,
        ColorType,
        const WIDTH: usize,
        const HEIGHT: usize,
        const CHAIN_LENGTH: usize,
        const COLOR_DEPTH: usize,
        const PER_FRAME_DENOMINATOR: u8,
        const WORDS_PER_PLANE: usize,
        const SCANLINES_PER_FRAME: usize,
        const BITMAP_ELEMENTS: usize,
    > DrawTarget
    for RgbMatrix<
        'a,
        ColorType,
        WIDTH,
        HEIGHT,
        CHAIN_LENGTH,
        COLOR_DEPTH,
        PER_FRAME_DENOMINATOR,
        WORDS_PER_PLANE,
        SCANLINES_PER_FRAME,
        BITMAP_ELEMENTS,
    >
where
    ColorType: PixelColor + Color<COLOR_DEPTH>,
{
    type Color = ColorType;

    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels {
            if coord.x >= 0 && coord.y >= 0 {
                // Ignore any errors
                let _ = self.set_pixel(coord.x as usize, coord.y as usize, color);
            }
        }
        Ok(())
    }
}
