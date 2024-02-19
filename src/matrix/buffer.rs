use alloc::boxed::Box;
use alloc::vec;

use esp32s3_hal::dma::DmaDescriptor;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FrameConfiguration<H, W> {
    height: H,
    width: W,
    chain_length: usize,
    color_depth: u8,
    per_frame_denominator: u8,
}

impl Default for FrameConfiguration<Option<usize>, Option<usize>> {
    fn default() -> Self {
        Self {
            height: Option::None,
            width: Option::None,
            chain_length: Self::DEFAULT_CHAIN_LENGTH,
            color_depth: Self::DEFAULT_COLOR_DEPTH,
            per_frame_denominator: Self::DEFAULT_PER_FRAME_DENOMINATOR,
        }
    }
}

impl<H: Copy, W: Copy> FrameConfiguration<H, W> {
    const DEFAULT_CHAIN_LENGTH: usize = 1;
    const DEFAULT_COLOR_DEPTH: u8 = 8;
    const DEFAULT_PER_FRAME_DENOMINATOR: u8 = 8;

    pub fn height(&self) -> H {
        self.height
    }

    pub fn with_height(&self, height: usize) -> FrameConfiguration<usize, W> {
        FrameConfiguration {
            height,
            width: self.width,
            chain_length: self.chain_length,
            color_depth: self.color_depth,
            per_frame_denominator: self.per_frame_denominator,
        }
    }

    pub fn width(&self) -> W {
        self.width
    }

    pub fn with_width(&self, width: usize) -> FrameConfiguration<H, usize> {
        FrameConfiguration {
            height: self.height,
            width: width,
            chain_length: self.chain_length,
            color_depth: self.color_depth,
            per_frame_denominator: self.per_frame_denominator,
        }
    }

    pub fn chain_length(&self) -> usize {
        self.chain_length
    }

    pub fn with_chain_length(&self, chain_length: usize) -> Self {
        Self {
            height: self.height,
            width: self.width,
            chain_length: chain_length,
            color_depth: self.color_depth,
            per_frame_denominator: self.per_frame_denominator,
        }
    }

    pub fn color_depth(&self) -> u8 {
        self.color_depth
    }

    pub fn with_color_depth(&self, color_depth: u8) -> Self {
        Self {
            height: self.height,
            width: self.width,
            chain_length: self.chain_length,
            color_depth: color_depth,
            per_frame_denominator: self.per_frame_denominator,
        }
    }

    pub fn per_frame_denominator(&self) -> u8 {
        self.per_frame_denominator
    }

    pub fn with_per_frame_denominator(&self, denominator: u8) -> Self {
        Self {
            height: self.height,
            width: self.width,
            chain_length: self.chain_length,
            color_depth: self.color_depth,
            per_frame_denominator: denominator,
        }
    }
}

impl FrameConfiguration<usize, usize> {
    pub fn new(height: usize, width: usize) -> Self {
        Self {
            height,
            width,
            chain_length: Self::DEFAULT_CHAIN_LENGTH,
            color_depth: Self::DEFAULT_COLOR_DEPTH,
            per_frame_denominator: Self::DEFAULT_PER_FRAME_DENOMINATOR,
        }
    }

    pub(crate) fn words_per_scanline(&self) -> usize {
        let pixels_per_row = self.width * self.chain_length;
        let rows_per_scanline = self.height / (self.per_frame_denominator as usize);
        // Each bit of color depth needs a separate word of storage as we're using BCD
        let pixels_per_scanline = pixels_per_row * (self.color_depth as usize) * rows_per_scanline;
        // Each word already encodes 2 pixels
        pixels_per_scanline / 2
    }

    pub(crate) fn scanlines_per_frame(&self) -> usize {
        let rows_per_scanline = self.height / (self.per_frame_denominator as usize);
        self.height / rows_per_scanline
    }

    pub(crate) fn words_per_frame(&self) -> usize {
        self.words_per_scanline() * self.scanlines_per_frame()
    }

    pub fn allocate_descriptors(&self) -> Box<[DmaDescriptor]> {
        // Allocate the descriptors for the buffers.
        // Quoting from the documentation:
        // Descriptors should be sized as (CHUNK_SIZE + 4091) / 4092. I.e., to transfer buffers of
        // size 1..=4092, you need 1 descriptor.
        let frame_bytes = self.words_per_frame() * core::mem::size_of::<u16>();
        vec![DmaDescriptor::EMPTY; (frame_bytes + 4091) / 4092].into_boxed_slice()
    }
}

pub struct FrameBuffer(pub(crate) Box<[u16]>);

impl FrameBuffer {
    pub fn new(frame_config: &FrameConfiguration<usize, usize>) -> Self {
        // Allocate all of the data for a frame in one contiguous buffer. Other libraries are
        // creating their DMA descriptors manually, but in Rust land we rely on ~abstractions~ to
        // handle that for us (hopefully it works).
        let buffer = vec![0u16; frame_config.words_per_frame()];
        Self(buffer.into_boxed_slice())
    }
}
