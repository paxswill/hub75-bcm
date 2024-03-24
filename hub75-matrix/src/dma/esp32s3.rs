use esp32s3_hal::clock::Clocks;
use esp32s3_hal::dma::{
    self, Channel, ChannelTx, ChannelTypes, DmaDescriptor, DmaError, DmaPeripheral, DmaPriority,
    LcdCamPeripheral, RegisterAccess, Tx, TxChannel,
};
use esp32s3_hal::peripheral::{Peripheral, PeripheralRef};
use esp32s3_hal::peripherals::LCD_CAM;
use fugit::HertzU32;

use crate::util::Sealed;
use crate::{const_check, const_not_zero};

use super::buffer::FrameBuffer;
use super::clock_divider::calculate_clkm;
use super::config::{MatrixConfig, MatrixPins};

pub trait MatrixChannelCreator<C: ChannelTypes>: Sealed {
    fn configure_lcd_channel<'a>(self, tx_descriptors: &'a mut [DmaDescriptor]) -> Channel<'a, C>;
}

macro_rules! impl_lcd_channel_creator {
    ($channel_creator:ty, $channel_type:ty) => {
        impl Sealed for $channel_creator {}
        impl MatrixChannelCreator<$channel_type> for $channel_creator {
            fn configure_lcd_channel<'a>(
                self,
                tx_descriptors: &'a mut [DmaDescriptor],
            ) -> Channel<'a, $channel_type> {
                self.configure(false, tx_descriptors, &mut [], DmaPriority::Priority0)
            }
        }
    };
}

impl_lcd_channel_creator!(dma::ChannelCreator0, dma::Channel0);
impl_lcd_channel_creator!(dma::ChannelCreator1, dma::Channel1);
impl_lcd_channel_creator!(dma::ChannelCreator2, dma::Channel2);
impl_lcd_channel_creator!(dma::ChannelCreator3, dma::Channel3);
impl_lcd_channel_creator!(dma::ChannelCreator4, dma::Channel4);

pub struct Transfer<
    'a,
    'd,
    TX: Tx,
    P,
    const WIDTH: usize,
    const HEIGHT: usize,
    const CHAIN_LENGTH: usize,
    const COLOR_DEPTH: usize,
    const PER_FRAME_DENOMINATOR: u8,
    const WORDS_PER_PLANE: usize,
    const SCANLINES_PER_FRAME: usize,
> {
    matrix_dma: MatrixDma<
        'd,
        TX,
        P,
        WIDTH,
        HEIGHT,
        CHAIN_LENGTH,
        COLOR_DEPTH,
        PER_FRAME_DENOMINATOR,
        WORDS_PER_PLANE,
        SCANLINES_PER_FRAME,
    >,
    frame_buffer: &'a FrameBuffer<
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
        'd,
        TX: Tx,
        P,
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
        'd,
        TX,
        P,
        WIDTH,
        HEIGHT,
        CHAIN_LENGTH,
        COLOR_DEPTH,
        PER_FRAME_DENOMINATOR,
        WORDS_PER_PLANE,
        SCANLINES_PER_FRAME,
    >
{
    pub fn stop(
        self,
    ) -> Result<
        (
            MatrixDma<
                'd,
                TX,
                P,
                WIDTH,
                HEIGHT,
                CHAIN_LENGTH,
                COLOR_DEPTH,
                PER_FRAME_DENOMINATOR,
                WORDS_PER_PLANE,
                SCANLINES_PER_FRAME,
            >,
            &'a FrameBuffer<
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
            DmaError,
            MatrixDma<
                'd,
                TX,
                P,
                WIDTH,
                HEIGHT,
                CHAIN_LENGTH,
                COLOR_DEPTH,
                PER_FRAME_DENOMINATOR,
                WORDS_PER_PLANE,
                SCANLINES_PER_FRAME,
            >,
            &'a FrameBuffer<
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
        // TODO Maybe add the interrup handler stuff ESP32-HUB75-MatrixPanel-I2S is doing?
        self.matrix_dma.lcd_cam.lcd_user().modify(|_, w| {
            w.lcd_reset()
                .set_bit()
                .lcd_update()
                .set_bit()
                .lcd_start()
                .clear_bit()
        });
        self.matrix_dma
            .lcd_cam
            .lc_dma_int_clr()
            .write(|w| w.lcd_trans_done_int_clr().clear_bit());
        // Wait for the DMA transfer to end.
        // TODO: This may not actually work...
        let dma_int_raw = self.matrix_dma.lcd_cam.lc_dma_int_raw();
        while dma_int_raw.read().lcd_trans_done_int_raw().bit_is_clear() {}

        if self.matrix_dma.tx_channel.has_error() {
            Err((
                DmaError::DescriptorError,
                self.matrix_dma,
                self.frame_buffer,
            ))
        } else {
            Ok((self.matrix_dma, self.frame_buffer))
        }
    }
}

pub struct MatrixDma<
    'd,
    TX,
    P,
    const WIDTH: usize,
    const HEIGHT: usize,
    const CHAIN_LENGTH: usize,
    const COLOR_DEPTH: usize,
    const PER_FRAME_DENOMINATOR: u8,
    const WORDS_PER_PLANE: usize,
    const SCANLINES_PER_FRAME: usize,
> {
    lcd_cam: PeripheralRef<'d, LCD_CAM>,

    tx_channel: TX,

    pins: P,

    config: MatrixConfig<WIDTH, HEIGHT, CHAIN_LENGTH, COLOR_DEPTH, PER_FRAME_DENOMINATOR>,
}

impl<
        'd,
        T,
        R,
        P,
        const WIDTH: usize,
        const HEIGHT: usize,
        const CHAIN_LENGTH: usize,
        const COLOR_DEPTH: usize,
        const PER_FRAME_DENOMINATOR: u8,
        const WORDS_PER_PLANE: usize,
        const SCANLINES_PER_FRAME: usize,
    >
    MatrixDma<
        'd,
        ChannelTx<'d, T, R>,
        P,
        WIDTH,
        HEIGHT,
        CHAIN_LENGTH,
        COLOR_DEPTH,
        PER_FRAME_DENOMINATOR,
        WORDS_PER_PLANE,
        SCANLINES_PER_FRAME,
    >
where
    T: TxChannel<R>,
    R: ChannelTypes + RegisterAccess,
    R::P: LcdCamPeripheral,
    P: MatrixPins,
{
    const_not_zero!(WIDTH, usize);
    const_not_zero!(HEIGHT, usize);
    const_not_zero!(CHAIN_LENGTH, usize);
    const_not_zero!(COLOR_DEPTH, usize);
    const_not_zero!(PER_FRAME_DENOMINATOR, u8);

    pub fn create<C, CC>(
        lcd_cam: impl Peripheral<P = LCD_CAM> + 'd,
        mut pins: P,
        frequency: HertzU32,
        config: MatrixConfig<WIDTH, HEIGHT, CHAIN_LENGTH, COLOR_DEPTH, PER_FRAME_DENOMINATOR>,
        clocks: &Clocks,
        channel_creator: CC,
        tx_descriptors: &'d mut [DmaDescriptor],
    ) -> Self
    where
        CC: MatrixChannelCreator<C>,
        C: ChannelTypes<Tx<'d> = ChannelTx<'d, T, R>>,
    {
        // Force the compiler to evaluate all the const checks
        let _ = Self::WIDTH;
        let _ = Self::HEIGHT;
        let _ = Self::CHAIN_LENGTH;
        let _ = Self::COLOR_DEPTH;
        let _ = Self::PER_FRAME_DENOMINATOR;
        let _ = Self::WORDS_PER_PLANE;
        let _ = Self::SCANLINES_PER_FRAME;

        // Check that we've been given enough descriptors

        // Due to https://www.espressif.com/sites/default/files/documentation/esp32-s3_errata_en.pdf
        // the LCD_PCLK divider must be at least 2. To make up for this the user
        // provided frequency is doubled to match.

        let (i, divider) = calculate_clkm(
            (frequency.to_Hz() * 2) as _,
            &[
                clocks.xtal_clock.to_Hz() as _,
                clocks.cpu_clock.to_Hz() as _,
                clocks.crypto_pwm_clock.to_Hz() as _,
            ],
        );

        lcd_cam.lcd_clock().write(|w| {
            // Force enable the clock for all configuration registers.
            w.clk_en()
                .set_bit()
                .lcd_clk_sel()
                .variant((i + 1) as _)
                .lcd_clkm_div_num()
                .variant(divider.div_num as _)
                .lcd_clkm_div_b()
                .variant(divider.div_b as _)
                .lcd_clkm_div_a()
                .variant(divider.div_a as _)
                // LCD_PCLK = LCD_CLK / 2
                .lcd_clk_equ_sysclk()
                .clear_bit()
                .lcd_clkcnt_n()
                .variant(2 - 1) // Must not be 0.
                // Pixel Clock starts low, then moves high
                .lcd_ck_idle_edge()
                .clear_bit()
                // Pixel Clock idles low
                .lcd_ck_out_edge()
                .clear_bit()
        });

        // Not RGB mode, as we're driving the pins in a weird fasion that's closer to i8080 mode
        lcd_cam
            .lcd_ctrl()
            .write(|w| w.lcd_rgb_mode_en().clear_bit());

        // And because we're not passing in RGB data, we don't want the YUV conversion hardware
        // changing anything.
        lcd_cam
            .lcd_rgb_yuv()
            .write(|w| w.lcd_conv_bypass().clear_bit());

        lcd_cam.lcd_user().modify(|_, w| {
            // Don't change the bit order, part 1
            w.lcd_8bits_order()
                .bit(false)
                // Don't change the bit order, part 2
                .lcd_bit_order()
                .clear_bit()
                // Don't change the byte order
                .lcd_byte_order()
                .clear_bit()
                // We're clocking out 2 bytes at a time
                .lcd_2byte_en()
                .set_bit()
                // We need dummy cycles
                .lcd_dummy()
                .set_bit()
                // We need 2 dummy cycles
                .lcd_dummy_cyclelen()
                .variant(2)
        });

        lcd_cam.lcd_misc().write(|w| {
            // Set the threshold for Async Tx FIFO full event. (5 bits)
            w.lcd_afifo_threshold_num()
                .variant(0)
                // Total of two setup cycles (this value + 1)
                // Configure the setup cycles in LCD non-RGB mode. Setup cycles
                // expected = this value + 1. (6 bit)
                .lcd_vfk_cyclelen()
                .variant(1)
                // Same as setup time, 2 hold cycles (this value + 1)
                .lcd_vbk_cyclelen()
                .variant(1)
                // Do not auto-frame data.
                .lcd_next_frame_en()
                .clear_bit()
                // Enable blank region when LCD sends data out.
                // Needed to work around a design flaw in the ESP32-S3:
                // https://esp32.com/viewtopic.php?t=24459&start=60#p91835
                .lcd_bk_en()
                .set_bit()
                // We don't need any of the clock edge congiurations
                .lcd_cd_data_set()
                .clear_bit()
                .lcd_cd_dummy_set()
                .clear_bit()
                .lcd_cd_cmd_set()
                .clear_bit()
                .lcd_cd_idle_edge()
                .clear_bit()
        });

        lcd_cam
            .lcd_dly_mode()
            // No delay mode
            .write(|w| w.lcd_cd_mode().variant(0));

        // No delay mode for all output pins
        lcd_cam.lcd_data_dout_mode().write(|w| {
            w.dout0_mode()
                .variant(0)
                .dout1_mode()
                .variant(0)
                .dout2_mode()
                .variant(0)
                .dout3_mode()
                .variant(0)
                .dout4_mode()
                .variant(0)
                .dout5_mode()
                .variant(0)
                .dout6_mode()
                .variant(0)
                .dout7_mode()
                .variant(0)
                .dout8_mode()
                .variant(0)
                .dout9_mode()
                .variant(0)
                .dout10_mode()
                .variant(0)
                .dout11_mode()
                .variant(0)
                .dout12_mode()
                .variant(0)
                .dout13_mode()
                .variant(0)
                .dout14_mode()
                .variant(0)
                .dout15_mode()
                .variant(0)
        });

        pins.configure();

        let channel = channel_creator.configure_lcd_channel(tx_descriptors);
        R::init_channel();

        Self {
            lcd_cam,
            tx_channel: channel.tx,
            pins,
            config,
        }
    }

    /// Start a continuous DMA transfer to the RGB matrix.
    ///
    /// Safety: The memory referred to by the `frame_buffer`argument cannot be written to while the
    /// DMA transfer is in progress. If the lifetime of the `frame_buffer` argument is `\`static`,
    /// this is guaranteed; but if it is any other lifetime it is possible to `core::mem::forget()`
    /// the `Transfer`, which would skip the normal stop of the ongoing transfer.
    pub unsafe fn start_reference<'a>(
        mut self,
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
            'd,
            ChannelTx<'d, T, R>,
            P,
            WIDTH,
            HEIGHT,
            CHAIN_LENGTH,
            COLOR_DEPTH,
            PER_FRAME_DENOMINATOR,
            WORDS_PER_PLANE,
            SCANLINES_PER_FRAME,
        >,
        (
            DmaError,
            Self,
            &'a FrameBuffer<
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
        // Reset operating registers to known state
        self.lcd_cam.lcd_user().modify(|_, w| {
            w.lcd_reset()
                .set_bit()
                .lcd_cmd()
                .clear_bit()
                // We're going to just keep looping everything
                .lcd_always_out_en()
                .set_bit()
                // We're getting ready to start pumping out data
                .lcd_dout()
                .set_bit()
        });
        self.lcd_cam
            .lcd_misc()
            .modify(|_, w| w.lcd_afifo_reset().set_bit());

        // Start the DMA transfer
        let dma_res = self
            .tx_channel
            .tx_impl
            .prepare_segmented_transfer_without_start(
                self.tx_channel.descriptors,
                true,
                DmaPeripheral::LcdCam,
                frame_buffer.buffer_iter().map(|buf| {
                    let ptr_range = buf.as_ptr_range();
                    // Safety:
                    // From the documentation from byte_offset_from():
                    // This is purely a convenience for casting to a u8 pointer and using offset_from
                    // on it. See that method for documentation and safety requirements.
                    //
                    // From byte_offset():
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
                }),
            );

        Ok(Transfer {
            matrix_dma: self,
            frame_buffer: frame_buffer,
        })
    }

    pub fn start(
        mut self,
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
            'd,
            ChannelTx<'d, T, R>,
            P,
            WIDTH,
            HEIGHT,
            CHAIN_LENGTH,
            COLOR_DEPTH,
            PER_FRAME_DENOMINATOR,
            WORDS_PER_PLANE,
            SCANLINES_PER_FRAME,
        >,
        (
            DmaError,
            Self,
            &'static FrameBuffer<
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
}
