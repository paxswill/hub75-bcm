use esp32s3_hal::clock::Clocks;
use esp32s3_hal::dma::{
    self, Channel, ChannelTx, ChannelTypes, DmaDescriptor, DmaError, DmaPeripheral, DmaPriority,
    LcdCamPeripheral, RegisterAccess, Tx, TxChannel, TxPrivate,
};
use esp32s3_hal::gpio::{DriveStrength, OutputPin, OutputSignal};
use esp32s3_hal::lcd_cam::lcd::Lcd;
use esp32s3_hal::lcd_cam::LcdCam;
use esp32s3_hal::peripheral::{Peripheral, PeripheralRef};
use esp32s3_hal::peripherals::LCD_CAM;
use esp32s3_hal::system;
use fugit::HertzU32;

use crate::util::Sealed;
use crate::{const_check, const_not_zero};

use crate::buffer::FrameBuffer;
use crate::clock_divider::calculate_clkm;
use crate::config::MatrixConfig;

use super::{MatrixDma, Transfer};

pub trait MatrixPins: Sealed {
    fn configure(&mut self);
}

pub struct Pins<
    'd,
    Red1,
    Green1,
    Blue1,
    Red2,
    Green2,
    Blue2,
    AddressA,
    AddressB,
    AddressC,
    AddressD,
    AddressE,
    OutputEnable,
    Latch,
    PixelClock,
> {
    red_1: PeripheralRef<'d, Red1>,
    green_1: PeripheralRef<'d, Green1>,
    blue_1: PeripheralRef<'d, Blue1>,
    red_2: PeripheralRef<'d, Red2>,
    green_2: PeripheralRef<'d, Green2>,
    blue_2: PeripheralRef<'d, Blue2>,
    address_a: PeripheralRef<'d, AddressA>,
    address_b: PeripheralRef<'d, AddressB>,
    address_c: PeripheralRef<'d, AddressC>,
    // Address lines D and E sometimes aren't needed. We can typically require at least 3 address
    // lines, as that's typically the minimum needed to drive at least 16 rows.
    address_d: Option<PeripheralRef<'d, AddressD>>,
    address_e: Option<PeripheralRef<'d, AddressE>>,
    output_enable: PeripheralRef<'d, OutputEnable>,
    latch: PeripheralRef<'d, Latch>,
    clock: PeripheralRef<'d, PixelClock>,
}

impl<
        'd,
        Red1,
        Green1,
        Blue1,
        Red2,
        Green2,
        Blue2,
        AddressA,
        AddressB,
        AddressC,
        AddressD,
        AddressE,
        OutputEnable,
        Latch,
        PixelClock,
    >
    Pins<
        'd,
        Red1,
        Green1,
        Blue1,
        Red2,
        Green2,
        Blue2,
        AddressA,
        AddressB,
        AddressC,
        AddressD,
        AddressE,
        OutputEnable,
        Latch,
        PixelClock,
    >
{
    const DEFAULT_DRIVE_STRENGTH: DriveStrength = DriveStrength::I40mA;

    pub fn new(
        red_1: impl Peripheral<P = Red1> + 'd,
        green_1: impl Peripheral<P = Green1> + 'd,
        blue_1: impl Peripheral<P = Blue1> + 'd,
        red_2: impl Peripheral<P = Red2> + 'd,
        green_2: impl Peripheral<P = Green2> + 'd,
        blue_2: impl Peripheral<P = Blue2> + 'd,
        address_a: impl Peripheral<P = AddressA> + 'd,
        address_b: impl Peripheral<P = AddressB> + 'd,
        address_c: impl Peripheral<P = AddressC> + 'd,
        address_d: Option<impl Peripheral<P = AddressD> + 'd>,
        address_e: Option<impl Peripheral<P = AddressE> + 'd>,
        output_enable: impl Peripheral<P = OutputEnable> + 'd,
        latch: impl Peripheral<P = Latch> + 'd,
        clock: impl Peripheral<P = PixelClock> + 'd,
    ) -> Self {
        Self {
            red_1: red_1.into_ref(),
            green_1: green_1.into_ref(),
            blue_1: blue_1.into_ref(),
            red_2: red_2.into_ref(),
            green_2: green_2.into_ref(),
            blue_2: blue_2.into_ref(),
            address_a: address_a.into_ref(),
            address_b: address_b.into_ref(),
            address_c: address_c.into_ref(),
            address_d: address_d.map(|p| p.into_ref()),
            address_e: address_e.map(|p| p.into_ref()),
            output_enable: output_enable.into_ref(),
            latch: latch.into_ref(),
            clock: clock.into_ref(),
        }
    }
}

impl<
        'd,
        Red1,
        Green1,
        Blue1,
        Red2,
        Green2,
        Blue2,
        AddressA,
        AddressB,
        AddressC,
        AddressD,
        AddressE,
        OutputEnable,
        Latch,
        PixelClock,
    > Sealed
    for Pins<
        'd,
        Red1,
        Green1,
        Blue1,
        Red2,
        Green2,
        Blue2,
        AddressA,
        AddressB,
        AddressC,
        AddressD,
        AddressE,
        OutputEnable,
        Latch,
        PixelClock,
    >
{
}

impl<
        'd,
        Red1,
        Green1,
        Blue1,
        Red2,
        Green2,
        Blue2,
        AddressA,
        AddressB,
        AddressC,
        AddressD,
        AddressE,
        OutputEnable,
        Latch,
        PixelClock,
    > MatrixPins
    for Pins<
        'd,
        Red1,
        Green1,
        Blue1,
        Red2,
        Green2,
        Blue2,
        AddressA,
        AddressB,
        AddressC,
        AddressD,
        AddressE,
        OutputEnable,
        Latch,
        PixelClock,
    >
where
    Red1: OutputPin,
    Green1: OutputPin,
    Blue1: OutputPin,
    Red2: OutputPin,
    Green2: OutputPin,
    Blue2: OutputPin,
    AddressA: OutputPin,
    AddressB: OutputPin,
    AddressC: OutputPin,
    AddressD: OutputPin,
    AddressE: OutputPin,
    OutputEnable: OutputPin,
    Latch: OutputPin,
    PixelClock: OutputPin,
{
    fn configure(&mut self) {
        self.red_1
            .set_to_push_pull_output()
            .set_drive_strength(Self::DEFAULT_DRIVE_STRENGTH)
            .connect_peripheral_to_output(OutputSignal::LCD_DATA_0);
        self.green_1
            .set_to_push_pull_output()
            .set_drive_strength(Self::DEFAULT_DRIVE_STRENGTH)
            .connect_peripheral_to_output(OutputSignal::LCD_DATA_1);
        self.blue_1
            .set_to_push_pull_output()
            .set_drive_strength(Self::DEFAULT_DRIVE_STRENGTH)
            .connect_peripheral_to_output(OutputSignal::LCD_DATA_2);
        self.red_2
            .set_to_push_pull_output()
            .set_drive_strength(Self::DEFAULT_DRIVE_STRENGTH)
            .connect_peripheral_to_output(OutputSignal::LCD_DATA_3);
        self.green_2
            .set_to_push_pull_output()
            .set_drive_strength(Self::DEFAULT_DRIVE_STRENGTH)
            .connect_peripheral_to_output(OutputSignal::LCD_DATA_4);
        self.blue_2
            .set_to_push_pull_output()
            .set_drive_strength(Self::DEFAULT_DRIVE_STRENGTH)
            .connect_peripheral_to_output(OutputSignal::LCD_DATA_5);
        self.address_a
            .set_to_push_pull_output()
            .set_drive_strength(Self::DEFAULT_DRIVE_STRENGTH)
            .connect_peripheral_to_output(OutputSignal::LCD_DATA_6);
        self.address_b
            .set_to_push_pull_output()
            .set_drive_strength(Self::DEFAULT_DRIVE_STRENGTH)
            .connect_peripheral_to_output(OutputSignal::LCD_DATA_7);
        self.address_c
            .set_to_push_pull_output()
            .set_drive_strength(Self::DEFAULT_DRIVE_STRENGTH)
            .connect_peripheral_to_output(OutputSignal::LCD_DATA_8);
        if let Some(address_d) = self.address_d.as_mut() {
            address_d
                .set_to_push_pull_output()
                .set_drive_strength(Self::DEFAULT_DRIVE_STRENGTH)
                .connect_peripheral_to_output(OutputSignal::LCD_DATA_9);
        }
        if let Some(address_e) = self.address_e.as_mut() {
            address_e
                .set_to_push_pull_output()
                .set_drive_strength(Self::DEFAULT_DRIVE_STRENGTH)
                .connect_peripheral_to_output(OutputSignal::LCD_DATA_10);
        }
        self.output_enable
            .set_to_push_pull_output()
            .set_drive_strength(Self::DEFAULT_DRIVE_STRENGTH)
            .connect_peripheral_to_output(OutputSignal::LCD_DATA_11);
        self.latch
            .set_to_push_pull_output()
            .set_drive_strength(Self::DEFAULT_DRIVE_STRENGTH)
            .connect_peripheral_to_output(OutputSignal::LCD_DATA_12);
        self.clock
            .set_to_push_pull_output()
            .set_drive_strength(Self::DEFAULT_DRIVE_STRENGTH)
            .connect_peripheral_to_output(OutputSignal::LCD_PCLK);
    }
}

impl<
        'd,
        Red1,
        Green1,
        Blue1,
        Red2,
        Green2,
        Blue2,
        AddressA,
        AddressB,
        AddressC,
        AddressD,
        AddressE,
        OutputEnable,
        Latch,
        PixelClock,
    > core::fmt::Debug
    for Pins<
        'd,
        Red1,
        Green1,
        Blue1,
        Red2,
        Green2,
        Blue2,
        AddressA,
        AddressB,
        AddressC,
        AddressD,
        AddressE,
        OutputEnable,
        Latch,
        PixelClock,
    >
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Pins").finish()
    }
}

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

pub struct Esp32s3Dma<
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
    lcd: Lcd<'d>,

    channel: TX,

    config: MatrixConfig<WIDTH, HEIGHT, CHAIN_LENGTH, COLOR_DEPTH, PER_FRAME_DENOMINATOR>,

    _pins: P,
}

impl<
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
    >
    Esp32s3Dma<
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

    pub const MIN_DESCRIPTOR_COUNT: usize = {
        ((Self::WORDS_PER_PLANE * core::mem::size_of::<u16>() + 4091) / 4092)
            * ((1 << (Self::COLOR_DEPTH)) - 1)
            * Self::SCANLINES_PER_FRAME
    };
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
    Esp32s3Dma<
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
    pub fn create<C, CC>(
        lcd: Lcd<'d>,
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

        lcd.lcd_cam.lcd_clock().write(|w| {
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
        lcd.lcd_cam
            .lcd_ctrl()
            .write(|w| w.lcd_rgb_mode_en().clear_bit());

        // And because we're not passing in RGB data, we don't want the YUV conversion hardware
        // changing anything.
        lcd.lcd_cam
            .lcd_rgb_yuv()
            .write(|w| w.lcd_conv_bypass().clear_bit());

        lcd.lcd_cam.lcd_user().modify(|_, w| {
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

        lcd.lcd_cam.lcd_misc().write(|w| {
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

        lcd.lcd_cam
            .lcd_dly_mode()
            // No delay mode
            .write(|w| w.lcd_cd_mode().variant(0));

        // No delay mode for all output pins
        lcd.lcd_cam.lcd_data_dout_mode().write(|w| {
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
            lcd,
            channel: channel.tx,
            config,
            _pins: pins,
        }
    }
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
        WIDTH,
        HEIGHT,
        CHAIN_LENGTH,
        COLOR_DEPTH,
        PER_FRAME_DENOMINATOR,
        WORDS_PER_PLANE,
        SCANLINES_PER_FRAME,
    >
    for Esp32s3Dma<
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
    type Error = DmaError;

    /// Start a continuous DMA transfer to the RGB matrix.
    ///
    /// Safety: The memory referred to by the `frame_buffer`argument cannot be written to while the
    /// DMA transfer is in progress. If the lifetime of the `frame_buffer` argument is `\`static`,
    /// this is guaranteed; but if it is any other lifetime it is possible to `core::mem::forget()`
    /// the `Transfer`, which would skip the normal stop of the ongoing transfer.
    unsafe fn start_reference<'a>(
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
            DmaError,
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
    > {
        // Reset operating registers to known state
        self.lcd.lcd_cam.lcd_user().modify(|_, w| {
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
        self.lcd
            .lcd_cam
            .lcd_misc()
            .modify(|_, w| w.lcd_afifo_reset().set_bit());

        // Start the DMA transfer
        let maybe_err = self
            .channel
            .tx_impl
            .prepare_segmented_transfer_without_start(
                self.channel.descriptors,
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
            )
            .and_then(|_| self.channel.tx_impl.start_transfer())
            .and_then(|_| {
                self.lcd
                    .lcd_cam
                    .lc_dma_int_clr()
                    .write(|w| w.lcd_trans_done_int_clr().set_bit());

                self.lcd
                    .lcd_cam
                    .lcd_user()
                    .modify(|_, w| w.lcd_update().set_bit().lcd_start().set_bit());
                Ok(())
            });
        match maybe_err {
            Ok(_) => Ok(Transfer {
                matrix_dma: self,
                frame_buffer: frame_buffer,
            }),
            Err(err) => Err((err, self, frame_buffer)),
        }
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
    > {
        // TODO Maybe add the interrupt handler stuff ESP32-HUB75-MatrixPanel-I2S is doing?
        log::debug!("Stopping RGB matrix DMA transfer");
        transfer.matrix_dma.lcd.lcd_cam.lcd_user().modify(|_, w| {
            w
                //.lcd_reset()
                //.set_bit()
                //.lcd_update()
                //.set_bit()
                .lcd_start()
                .clear_bit()
        });
        log::trace!(
            "LCD_USER register: {:#034b}",
            transfer.matrix_dma.lcd.lcd_cam.lcd_user().read().bits()
        );
        transfer
            .matrix_dma
            .lcd
            .lcd_cam
            .lc_dma_int_clr()
            .write(|w| w.lcd_trans_done_int_clr().clear_bit());
        // Wait for the DMA transfer to end.
        // TODO: This may not actually work...
        let dma_int_raw = transfer.matrix_dma.lcd.lcd_cam.lc_dma_int_raw();
        while dma_int_raw.read().lcd_trans_done_int_raw().bit_is_clear() {
            log::trace!("RGB matrix DMA transfer still in progress");
        }

        if transfer.matrix_dma.channel.has_error() {
            Err((
                DmaError::DescriptorError,
                transfer.matrix_dma,
                transfer.frame_buffer,
            ))
        } else {
            Ok((transfer.matrix_dma, transfer.frame_buffer))
        }
    }
}

impl<
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
    > core::fmt::Debug
    for Esp32s3Dma<
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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Esp32s3Dma")
            .field("config", &self.config)
            .finish()
    }
}
