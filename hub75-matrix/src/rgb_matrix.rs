use core::iter;
use embedded_graphics_core::draw_target::DrawTarget;
use embedded_graphics_core::geometry::{OriginDimensions, Size};
use embedded_graphics_core::pixelcolor::PixelColor;
use embedded_graphics_core::Pixel;

use crate::{const_check, const_not_zero};

use super::buffer::FrameBuffer;
use super::color::Color;
use super::config::MatrixConfig;

enum MatrixError {
    OutOfBounds,
}

pub struct RgbMatrix<
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
}

impl<
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

    const WORDS_PER_PLANE: usize = const_check!(
        WORDS_PER_PLANE,
        WORDS_PER_PLANE
            == (
                // Divide by 2 because each word encodes two colors
                WIDTH * CHAIN_LENGTH * HEIGHT / (PER_FRAME_DENOMINATOR as usize) / 2
            ),
        "WORDS_PER_PLANE must equal WIDTH * CHAIN_LENGTH * HEIGHT / PER_FRAME_DENOMINATOR / 2"
    );

    const WORDS_PER_SCANLINE: usize = {
        let pixels_per_row = Self::WIDTH * Self::CHAIN_LENGTH;
        let rows_per_scanline = Self::HEIGHT / (Self::PER_FRAME_DENOMINATOR as usize);
        // Each bit of color depth needs a separate word of storage as we're using BCD
        let pixels_per_scanline = pixels_per_row * (Self::COLOR_DEPTH as usize) * rows_per_scanline;
        // Each word already encodes 2 pixels
        pixels_per_scanline / 2
    };

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
}

impl<
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
    ColorType: PartialEq,
{
    fn set_pixel(&mut self, x: usize, y: usize, new_color: ColorType) -> Result<(), MatrixError> {
        // Discard early out of bounds coordinates. We also can't use pattern matching here
        // because we're using const generics for all the bounds.
        // from within trait implementations.
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
        }
        self.pixel_buffer[y][panel_index][panel_x] = new_color;
        Ok(())
    }
}

impl<
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
        Self {
            config,
            pixel_buffer: [[[ColorType::default(); WIDTH]; CHAIN_LENGTH]; HEIGHT],
            dirty_bitmap: [0u32; BITMAP_ELEMENTS],
            brightness: Self::DEFAULT_BRIGHTNESS,
            brightness_dirty: false,
        }
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
        frame_buffer.set_control_bits(self.config.latch_blanking_count());
        frame_buffer.set_brightness_bits(self.config.latch_blanking_count(), self.brightness);
    }

    pub fn brightness(&self) -> u8 {
        self.brightness
    }

    pub fn set_brightness(&mut self, new_brightness: u8) {
        self.brightness_dirty = new_brightness != self.brightness;
        self.brightness = new_brightness;
    }

    pub fn flush(
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
        let pixel_buffer_iter = self
            .pixel_buffer
            .iter()
            .enumerate()
            .flat_map(|(y, panel)| panel.iter().zip(iter::repeat(y)))
            .enumerate()
            .flat_map(|(panel_index, (row, y))| {
                let row_offset = panel_index * Self::WIDTH;
                row.iter()
                    .enumerate()
                    .map(move |(row_index, color)| ((row_offset + row_index, y), color))
            });

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
