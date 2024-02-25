use esp32s3_hal::gpio::{DriveStrength, OutputPin, OutputSignal};
use esp32s3_hal::peripheral::{Peripheral, PeripheralRef};

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
