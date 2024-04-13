#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
enum BitOffsets {
    Red1 = 0,
    Green1,
    Blue1,
    Red2,
    Green2,
    Blue2,
    Latch,
    OutputEnable,
    AddressA,
    AddressB,
    AddressC,
    AddressD,
    AddressE,
}

impl BitOffsets {
    const ADDRESS_MASK: u16 = {
        Self::AddressA.bit_for()
            | Self::AddressB.bit_for()
            | Self::AddressC.bit_for()
            | Self::AddressD.bit_for()
            | Self::AddressE.bit_for()
    };

    #[inline]
    const fn bit_for(&self) -> u16 {
        1 << (*self as u8)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum MatrixPixel {
    One = 0,
    Two,
}

pub trait MatrixWord {
    /// Is the red bit for the given pixel set?
    fn red(&self, pixel: MatrixPixel) -> bool;

    /// Is the red bit for the given pixel set?
    fn green(&self, pixel: MatrixPixel) -> bool;

    /// Is the red bit for the given pixel set?
    fn blue(&self, pixel: MatrixPixel) -> bool;

    /// Is the latch (LAT) bit on?
    fn latch(&self) -> bool;

    /// Is the output enable (OE) bit set?
    fn output_enable(&self) -> bool;

    /// The address line selected.
    ///
    /// This is a 5-bit value (0-32).
    fn address(&self) -> u8;
}

pub trait MatrixWordMut: MatrixWord {
    /// Set the red bit for the given pixel to the given value.
    fn set_red_to(&mut self, pixel: MatrixPixel, value: bool);

    // Set the red bit for the given pixel.
    fn set_red(&mut self, pixel: MatrixPixel) {
        self.set_red_to(pixel, true)
    }

    /// Clear the red bit for the given pixel.
    fn clear_red(&mut self, pixel: MatrixPixel) {
        self.set_red_to(pixel, false)
    }

    /// Set the green bit for the given pixel to the given value.
    fn set_green_to(&mut self, pixel: MatrixPixel, value: bool);

    // Set the green bit for the given pixel.
    fn set_green(&mut self, pixel: MatrixPixel) {
        self.set_green_to(pixel, true)
    }

    /// Clear the green bit for the given pixel.
    fn clear_green(&mut self, pixel: MatrixPixel) {
        self.set_green_to(pixel, false)
    }

    /// Set the blue bit for the given pixel to the given value.
    fn set_blue_to(&mut self, pixel: MatrixPixel, value: bool);

    // Set the blue bit for the given pixel.
    fn set_blue(&mut self, pixel: MatrixPixel) {
        self.set_blue_to(pixel, true)
    }

    /// Clear the blue bit for the given pixel.
    fn clear_blue(&mut self, pixel: MatrixPixel) {
        self.set_blue_to(pixel, false)
    }

    /// Set the latch (LAT) bit to the given value.
    fn set_latch_to(&mut self, value: bool);

    /// Set the latch (LAT) bit on.
    fn set_latch(&mut self) {
        self.set_latch_to(true)
    }

    /// Reset the latch (LAT) bit to 0.
    fn clear_latch(&mut self) {
        self.set_latch_to(false)
    }

    /// Set the output enable (OE) bit to the given value.
    fn set_output_enable_to(&mut self, value: bool);

    /// Set the output enable (OE) bit on.
    fn set_output_enable(&mut self) {
        self.set_output_enable_to(true)
    }

    /// Reset the output enable (OE) bit to 0.
    fn clear_output_enable(&mut self) {
        self.set_output_enable_to(false)
    }

    /// Set the address bits to the given value.
    ///
    /// Any value greater than 31 is truncated to 31.
    fn set_address(&mut self, address: u8);
}

impl MatrixWord for u16 {
    fn red(&self, pixel: MatrixPixel) -> bool {
        let mask = match pixel {
            MatrixPixel::One => BitOffsets::Red1,
            MatrixPixel::Two => BitOffsets::Red2,
        }
        .bit_for();
        self & mask != 0
    }

    fn green(&self, pixel: MatrixPixel) -> bool {
        let mask = match pixel {
            MatrixPixel::One => BitOffsets::Green1,
            MatrixPixel::Two => BitOffsets::Green2,
        }
        .bit_for();
        self & mask != 0
    }

    fn blue(&self, pixel: MatrixPixel) -> bool {
        let mask = match pixel {
            MatrixPixel::One => BitOffsets::Blue1,
            MatrixPixel::Two => BitOffsets::Blue2,
        }
        .bit_for();
        self & mask != 0
    }

    fn latch(&self) -> bool {
        self & BitOffsets::Latch.bit_for() != 0
    }

    fn output_enable(&self) -> bool {
        self & BitOffsets::OutputEnable.bit_for() != 0
    }

    fn address(&self) -> u8 {
        ((self & BitOffsets::ADDRESS_MASK) >> BitOffsets::AddressA as u16) as u8
    }
}

impl MatrixWordMut for u16 {
    fn set_red_to(&mut self, pixel: MatrixPixel, value: bool) {
        let mask = match pixel {
            MatrixPixel::One => BitOffsets::Red1,
            MatrixPixel::Two => BitOffsets::Red2,
        }
        .bit_for();
        if value {
            *self |= mask;
        } else {
            *self &= !mask;
        }
    }

    fn set_green_to(&mut self, pixel: MatrixPixel, value: bool) {
        let mask = match pixel {
            MatrixPixel::One => BitOffsets::Green1,
            MatrixPixel::Two => BitOffsets::Green2,
        }
        .bit_for();
        if value {
            *self |= mask;
        } else {
            *self &= !mask;
        }
    }

    fn set_blue_to(&mut self, pixel: MatrixPixel, value: bool) {
        let mask = match pixel {
            MatrixPixel::One => BitOffsets::Blue1,
            MatrixPixel::Two => BitOffsets::Blue2,
        }
        .bit_for();
        if value {
            *self |= mask;
        } else {
            *self &= !mask;
        }
    }

    fn set_latch_to(&mut self, value: bool) {
        if value {
            self.set_latch()
        } else {
            self.clear_latch()
        }
    }

    fn set_latch(&mut self) {
        *self |= BitOffsets::Latch.bit_for()
    }

    fn clear_latch(&mut self) {
        *self &= !BitOffsets::Latch.bit_for()
    }

    fn set_output_enable_to(&mut self, value: bool) {
        if value {
            self.set_output_enable()
        } else {
            self.clear_output_enable()
        }
    }

    fn set_output_enable(&mut self) {
        *self |= BitOffsets::OutputEnable.bit_for()
    }

    fn clear_output_enable(&mut self) {
        *self &= !BitOffsets::OutputEnable.bit_for()
    }

    fn set_address(&mut self, address: u8) {
        let address = address.min(31) as u16;
        *self &= !BitOffsets::ADDRESS_MASK;
        *self |= address << BitOffsets::AddressA as u16;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn red_1_not_set() {
        assert!(!0u16.red(MatrixPixel::One))
    }

    #[test]
    fn red_1_set() {
        assert!(1u16.red(MatrixPixel::One))
    }

    #[test]
    fn set_red_1() {
        let mut val = 0u16;
        assert!(!val.red(MatrixPixel::One));
        val.set_red_to(MatrixPixel::One, false);
        assert_eq!(val, 0, "Bit changed when set to false when already unset");
        val.set_red_to(MatrixPixel::One, true);
        assert_eq!(val, 1u16, "Bit not changed to set after set method");
        val.set_red_to(MatrixPixel::One, false);
        assert_eq!(val, 0, "Bit not reset when set to false");
    }

    #[test]
    fn green_1_not_set() {
        assert!(!0u16.green(MatrixPixel::One))
    }

    #[test]
    fn green_1_set() {
        assert!(2u16.green(MatrixPixel::One))
    }

    #[test]
    fn set_green_1() {
        let mut val = 0u16;
        assert!(!val.green(MatrixPixel::One));
        val.set_green_to(MatrixPixel::One, false);
        assert_eq!(val, 0, "Bit changed when set to false when already unset");
        val.set_green_to(MatrixPixel::One, true);
        assert_eq!(val, 2u16, "Bit not changed to set after set method");
        val.set_green_to(MatrixPixel::One, false);
        assert_eq!(val, 0, "Bit not reset when set to false");
    }

    #[test]
    fn blue_1_not_set() {
        assert!(!0u16.blue(MatrixPixel::One))
    }

    #[test]
    fn blue_1_set() {
        assert!(4u16.blue(MatrixPixel::One))
    }

    #[test]
    fn set_blue_1() {
        let mut val = 0u16;
        assert!(!val.blue(MatrixPixel::One));
        val.set_blue_to(MatrixPixel::One, false);
        assert_eq!(val, 0, "Bit changed when set to false when already unset");
        val.set_blue_to(MatrixPixel::One, true);
        assert_eq!(val, 4u16, "Bit not changed to set after set method");
        val.set_blue_to(MatrixPixel::One, false);
        assert_eq!(val, 0, "Bit not reset when set to false");
    }

    #[test]
    fn red_2_not_set() {
        assert!(!0u16.red(MatrixPixel::Two))
    }

    #[test]
    fn red_2_set() {
        assert!(8u16.red(MatrixPixel::Two))
    }

    #[test]
    fn set_red_2() {
        let mut val = 0u16;
        assert!(!val.red(MatrixPixel::Two));
        val.set_red_to(MatrixPixel::Two, false);
        assert_eq!(val, 0, "Bit changed when set to false when already unset");
        val.set_red_to(MatrixPixel::Two, true);
        assert_eq!(val, 8u16, "Bit not changed to set after set method");
        val.set_red_to(MatrixPixel::Two, false);
        assert_eq!(val, 0, "Bit not reset when set to false");
    }

    #[test]
    fn green_2_not_set() {
        assert!(!0u16.green(MatrixPixel::Two))
    }

    #[test]
    fn green_2_set() {
        assert!(16u16.green(MatrixPixel::Two))
    }

    #[test]
    fn set_green_2() {
        let mut val = 0u16;
        assert!(!val.green(MatrixPixel::Two));
        val.set_green_to(MatrixPixel::Two, false);
        assert_eq!(val, 0, "Bit changed when set to false when already unset");
        val.set_green_to(MatrixPixel::Two, true);
        assert_eq!(val, 16u16, "Bit not changed to set after set method");
        val.set_green_to(MatrixPixel::Two, false);
        assert_eq!(val, 0, "Bit not reset when set to false");
    }

    #[test]
    fn blue_2_not_set() {
        assert!(!0u16.blue(MatrixPixel::Two))
    }

    #[test]
    fn blue_2_set() {
        assert!(32u16.blue(MatrixPixel::Two))
    }

    #[test]
    fn set_blue_2() {
        let mut val = 0u16;
        assert!(!val.blue(MatrixPixel::Two));
        val.set_blue_to(MatrixPixel::Two, false);
        assert_eq!(val, 0, "Bit changed when set to false when already unset");
        val.set_blue_to(MatrixPixel::Two, true);
        assert_eq!(val, 32u16, "Bit not changed to set after set method");
        val.set_blue_to(MatrixPixel::Two, false);
        assert_eq!(val, 0, "Bit not reset when set to false");
    }

    #[test]
    fn latch_not_set() {
        assert!(!0u16.latch())
    }

    #[test]
    fn latch_set() {
        assert!(64u16.latch())
    }

    #[test]
    fn set_latch() {
        let mut val = 0u16;
        assert!(!val.latch());
        val.set_latch_to(false);
        assert_eq!(val, 0, "Bit changed when set to false when already unset");
        val.set_latch_to(true);
        assert_eq!(val, 64u16, "Bit not changed to set after set method");
        val.set_latch_to(false);
        assert_eq!(val, 0, "Bit not reset when set to false");
    }

    #[test]
    fn output_enable_not_set() {
        assert!(!0u16.output_enable())
    }

    #[test]
    fn output_enable_set() {
        assert!(128u16.output_enable())
    }

    #[test]
    fn set_output_enable() {
        let mut val = 0u16;
        assert!(!val.output_enable());
        val.set_output_enable_to(false);
        assert_eq!(val, 0, "Bit changed when set to false when already unset");
        val.set_output_enable_to(true);
        assert_eq!(val, 128u16, "Bit not changed to set after set method");
        val.set_output_enable_to(false);
        assert_eq!(val, 0, "Bit not reset when set to false");
    }

    #[test]
    fn address_all() {
        for addr in 0u16..31 {
            let shifted = addr << 8;
            assert_eq!(
                shifted.address(),
                addr as u8,
                "Address {} not found for {:#06X}",
                addr,
                shifted
            );
        }
    }

    #[test]
    fn set_address_all() {
        for addr in 0u16..31 {
            let expected = addr << 8;
            let mut val = 0u16;
            val.set_address(addr as u8);
            assert_eq!(
                val, expected,
                "Newly set address is incorrect for address {}",
                addr
            );
        }
    }

    #[test]
    fn set_address_truncate() {
        let mut val = 0u16;
        val.set_address(100);
        let mut expected = 0u16;
        expected.set_address(31);
        assert_ne!(val, 0, "value did not change");
        assert_eq!(val, expected);
    }
}
