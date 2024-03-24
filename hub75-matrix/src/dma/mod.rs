use super::buffer::FrameBuffer;

#[cfg(feature = "esp32s3")]
pub mod esp32s3;

#[derive(Debug)]
pub struct Transfer<
    'a,
    M,
    const WIDTH: usize,
    const HEIGHT: usize,
    const CHAIN_LENGTH: usize,
    const COLOR_DEPTH: usize,
    const PER_FRAME_DENOMINATOR: u8,
    const WORDS_PER_PLANE: usize,
    const SCANLINES_PER_FRAME: usize,
> where
    M: MatrixDma<
        WIDTH,
        HEIGHT,
        CHAIN_LENGTH,
        COLOR_DEPTH,
        PER_FRAME_DENOMINATOR,
        WORDS_PER_PLANE,
        SCANLINES_PER_FRAME,
    >,
{
    matrix_dma: M,

    frame_buffer: &'a mut FrameBuffer<
        WIDTH,
        HEIGHT,
        CHAIN_LENGTH,
        COLOR_DEPTH,
        PER_FRAME_DENOMINATOR,
        WORDS_PER_PLANE,
        SCANLINES_PER_FRAME,
    >,
}

impl<
        'a,
        M,
        const WIDTH: usize,
        const HEIGHT: usize,
        const CHAIN_LENGTH: usize,
        const COLOR_DEPTH: usize,
        const PER_FRAME_DENOMINATOR: u8,
        const WORDS_PER_PLANE: usize,
        const SCANLINES_PER_FRAME: usize,
    >
    Transfer<
        'a,
        M,
        WIDTH,
        HEIGHT,
        CHAIN_LENGTH,
        COLOR_DEPTH,
        PER_FRAME_DENOMINATOR,
        WORDS_PER_PLANE,
        SCANLINES_PER_FRAME,
    >
where
    M: MatrixDma<
        WIDTH,
        HEIGHT,
        CHAIN_LENGTH,
        COLOR_DEPTH,
        PER_FRAME_DENOMINATOR,
        WORDS_PER_PLANE,
        SCANLINES_PER_FRAME,
    >,
{
    pub fn stop(
        self,
    ) -> Result<
        (
            M,
            &'a mut FrameBuffer<
                WIDTH,
                HEIGHT,
                CHAIN_LENGTH,
                COLOR_DEPTH,
                PER_FRAME_DENOMINATOR,
                WORDS_PER_PLANE,
                SCANLINES_PER_FRAME,
            >,
        ),
        (
            M::Error,
            M,
            &'a mut FrameBuffer<
                WIDTH,
                HEIGHT,
                CHAIN_LENGTH,
                COLOR_DEPTH,
                PER_FRAME_DENOMINATOR,
                WORDS_PER_PLANE,
                SCANLINES_PER_FRAME,
            >,
        ),
    > {
        M::stop(self)
    }
}

pub trait MatrixDma<
    const WIDTH: usize,
    const HEIGHT: usize,
    const CHAIN_LENGTH: usize,
    const COLOR_DEPTH: usize,
    const PER_FRAME_DENOMINATOR: u8,
    const WORDS_PER_PLANE: usize,
    const SCANLINES_PER_FRAME: usize,
>: Sized
{
    type Error;

    /// Start a continuous DMA transfer to the RGB matrix.
    ///
    /// Safety: The memory referred to by the `frame_buffer`argument cannot be written to while the
    /// DMA transfer is in progress. If the lifetime of the `frame_buffer` argument is `\`static`,
    /// this is guaranteed; but if it is any other lifetime it is possible to `core::mem::forget()`
    /// the `Transfer`, which would skip the normal stop of the ongoing transfer.
    unsafe fn start_reference<'a>(
        self,
        frame_buffer: &'a mut FrameBuffer<
            WIDTH,
            HEIGHT,
            CHAIN_LENGTH,
            COLOR_DEPTH,
            PER_FRAME_DENOMINATOR,
            WORDS_PER_PLANE,
            SCANLINES_PER_FRAME,
        >,
    ) -> Result<
        Transfer<
            'a,
            Self,
            WIDTH,
            HEIGHT,
            CHAIN_LENGTH,
            COLOR_DEPTH,
            PER_FRAME_DENOMINATOR,
            WORDS_PER_PLANE,
            SCANLINES_PER_FRAME,
        >,
        (
            Self::Error,
            Self,
            &'a mut FrameBuffer<
                WIDTH,
                HEIGHT,
                CHAIN_LENGTH,
                COLOR_DEPTH,
                PER_FRAME_DENOMINATOR,
                WORDS_PER_PLANE,
                SCANLINES_PER_FRAME,
            >,
        ),
    >;

    fn start(
        self,
        frame_buffer: &'static mut FrameBuffer<
            WIDTH,
            HEIGHT,
            CHAIN_LENGTH,
            COLOR_DEPTH,
            PER_FRAME_DENOMINATOR,
            WORDS_PER_PLANE,
            SCANLINES_PER_FRAME,
        >,
    ) -> Result<
        Transfer<
            'static,
            Self,
            WIDTH,
            HEIGHT,
            CHAIN_LENGTH,
            COLOR_DEPTH,
            PER_FRAME_DENOMINATOR,
            WORDS_PER_PLANE,
            SCANLINES_PER_FRAME,
        >,
        (
            Self::Error,
            Self,
            &'static mut FrameBuffer<
                WIDTH,
                HEIGHT,
                CHAIN_LENGTH,
                COLOR_DEPTH,
                PER_FRAME_DENOMINATOR,
                WORDS_PER_PLANE,
                SCANLINES_PER_FRAME,
            >,
        ),
    > {
        // Safety: `start()` is safe if the lifetime is `static, which is enforced by the function
        // signature.
        unsafe { self.start_reference(frame_buffer) }
    }

    fn stop<'a>(
        transfer: Transfer<
            'a,
            Self,
            WIDTH,
            HEIGHT,
            CHAIN_LENGTH,
            COLOR_DEPTH,
            PER_FRAME_DENOMINATOR,
            WORDS_PER_PLANE,
            SCANLINES_PER_FRAME,
        >,
    ) -> Result<
        (
            Self,
            &'a mut FrameBuffer<
                WIDTH,
                HEIGHT,
                CHAIN_LENGTH,
                COLOR_DEPTH,
                PER_FRAME_DENOMINATOR,
                WORDS_PER_PLANE,
                SCANLINES_PER_FRAME,
            >,
        ),
        (
            Self::Error,
            Self,
            &'a mut FrameBuffer<
                WIDTH,
                HEIGHT,
                CHAIN_LENGTH,
                COLOR_DEPTH,
                PER_FRAME_DENOMINATOR,
                WORDS_PER_PLANE,
                SCANLINES_PER_FRAME,
            >,
        ),
    >;
}
