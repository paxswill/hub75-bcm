//! RGB matrix setup and control.

mod buffer;

use esp32s3_hal::clock::Clocks;
use esp32s3_hal::dma::{
    self, Channel, ChannelTx, ChannelTypes, DmaDescriptor, DmaPriority, LcdCamPeripheral,
    RegisterAccess, TxChannel,
};
use esp32s3_hal::gpio::OutputPin;
use esp32s3_hal::lcd_cam::lcd::{i8080, ClockMode, DelayMode, Lcd, Phase, Polarity};
use esp32s3_hal::peripheral::Peripheral;

use fugit::RateExtU32;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
enum CurrentBuffer {
    #[default]
    A,
    B,
}

trait Sealed {}

pub trait LcdChannelCreator<C: ChannelTypes>: Sealed {
    fn configure_lcd_channel<'a>(self, tx_descriptors: &'a mut [DmaDescriptor]) -> Channel<'a, C>;
}

macro_rules! impl_lcd_channel_creator {
    ($channel_creator:ty, $channel_type:ty) => {
        impl Sealed for $channel_creator {}
        impl LcdChannelCreator<$channel_type> for $channel_creator {
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

pub struct LcdRgbMatrix<
    'd,
    TX,
    Red1,
    Blue1,
    Green1,
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
    NC1,
    NC2,
    NC3,
> {
    frame_config: buffer::FrameConfiguration<usize, usize>,
    lcd: Option<
        i8080::I8080<
            'd,
            TX,
            i8080::TxSixteenBits<
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
                NC1,
                NC2,
                NC3,
            >,
        >,
    >,
    next_buffer: Option<buffer::FrameBuffer>,
    last_buffer: Option<buffer::FrameBuffer>,
}

impl<
        'd,
        T,
        R,
        Red1,
        Blue1,
        Green1,
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
        NC1,
        NC2,
        NC3,
    >
    LcdRgbMatrix<
        'd,
        ChannelTx<'d, T, R>,
        Red1,
        Blue1,
        Green1,
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
        NC1,
        NC2,
        NC3,
    >
where
    T: TxChannel<R>,
    R: ChannelTypes + RegisterAccess,
    <R as ChannelTypes>::P: LcdCamPeripheral,
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
    NC1: OutputPin,
    NC2: OutputPin,
    NC3: OutputPin,
{
    pub fn new<PixelClock, C, NC4, CC>(
        frame_config: buffer::FrameConfiguration<usize, usize>,
        channel_creator: CC,
        tx_descriptors: &'d mut [DmaDescriptor],
        double_buffer: bool,
        lcd: Lcd<'d>,
        clocks: &Clocks<'_>,
        pin_red_1: impl Peripheral<P = Red1> + 'd,
        pin_blue_1: impl Peripheral<P = Blue1> + 'd,
        pin_green_1: impl Peripheral<P = Green1> + 'd,
        pin_red_2: impl Peripheral<P = Red2> + 'd,
        pin_blue_2: impl Peripheral<P = Blue2> + 'd,
        pin_green_2: impl Peripheral<P = Green2> + 'd,
        pin_address_a: impl Peripheral<P = AddressA> + 'd,
        pin_address_b: impl Peripheral<P = AddressB> + 'd,
        pin_address_c: impl Peripheral<P = AddressC> + 'd,
        pin_address_d: impl Peripheral<P = AddressD> + 'd,
        pin_address_e: impl Peripheral<P = AddressE> + 'd,
        pin_output_enable: impl Peripheral<P = OutputEnable> + 'd,
        pin_latch: impl Peripheral<P = Latch> + 'd,
        pin_clock: impl Peripheral<P = PixelClock> + 'd,
        // These don't care pins are needed until I guess the LCD-CAM peripheral is taught to
        // handle them optionally
        dont_care_1: impl Peripheral<P = NC1> + 'd,
        dont_care_2: impl Peripheral<P = NC2> + 'd,
        dont_care_3: impl Peripheral<P = NC3> + 'd,
        dont_care_4: impl Peripheral<P = NC4> + 'd,
    ) -> Self
    where
        PixelClock: OutputPin,
        NC4: OutputPin,
        CC: LcdChannelCreator<C>,
        C: ChannelTypes<Tx<'d> = ChannelTx<'d, T, R>>,
    {
        let pins = i8080::TxSixteenBits::new(
            pin_red_1,
            pin_green_1,
            pin_blue_1,
            pin_red_2,
            pin_green_2,
            pin_blue_2,
            pin_address_a,
            pin_address_b,
            pin_address_c,
            pin_address_d,
            pin_address_e,
            pin_output_enable,
            pin_latch,
            dont_care_1,
            dont_care_2,
            dont_care_3,
        );
        // Keeping with the same configuration as used by ESP32-HUB75-MatrixPanel-DMA
        let config = i8080::Config {
            // Pixel clock starts low, then moves high. The pixel clock should also idle low.
            clock_mode: ClockMode {
                polarity: Polarity::IdleLow,
                phase: Phase::ShiftLow,
            },
            // Default or minimum values for the rest
            setup_cycles: 1,
            hold_cycles: 1,
            cd_idle_edge: false,
            cd_cmd_edge: false,
            cd_dummy_edge: false,
            cd_data_edge: false,
            cd_mode: DelayMode::None,
            output_bit_mode: DelayMode::None,
        };
        // Allocate buffers
        let next_buffer = Some(buffer::FrameBuffer::new(&frame_config));
        let last_buffer = if double_buffer {
            Some(buffer::FrameBuffer::new(&frame_config))
        } else {
            None
        };
        // ESP32-HUB75-MatrixPanel-DMA does a lot of register setup ahead of time, but to keep
        // within the bounds of the HAL implementation we're going to rely on it setting them.
        // For documentation purposes, here's the register values we're not setting here:
        // 8-bit order, bit order, byte order, and 2-byte mode are set by the I8080 constructor
        // Dummy enable and dummy cycle count are implemented in send()/send_dma()
        // Disabling the command is also done in send()/send_dma() by setting Coomand::none
        // Configure the DMA channel
        let channel = channel_creator.configure_lcd_channel(tx_descriptors);

        let i8080_lcd = i8080::I8080::new(lcd, channel.tx, pins, 20u32.MHz(), config, clocks)
            .with_ctrl_pins(dont_care_4, pin_clock);
        Self {
            frame_config,
            lcd: Some(i8080_lcd),
            next_buffer,
            last_buffer,
        }
    }
}
