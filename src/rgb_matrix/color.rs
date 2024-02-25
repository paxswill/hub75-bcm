use embedded_graphics_core::pixelcolor::{Rgb555, Rgb666, Rgb888, RgbColor};

use super::buffer::ColorStorage;

pub trait Color<const DEPTH: usize> {
    type Storage: ColorStorage<DEPTH>;

    fn new<R: AsRef<Self::Storage>, G: AsRef<Self::Storage>, B: AsRef<Self::Storage>>(
        red: R,
        green: G,
        blue: B,
    ) -> Self;

    fn red(&self) -> Self::Storage;

    fn green(&self) -> Self::Storage;

    fn blue(&self) -> Self::Storage;
}

macro_rules! impl_pixel_color {
    ($pixel_type:ty, $color_depth:literal, $component_type:ty) => {
        impl Color<$color_depth> for $pixel_type {
            type Storage = $component_type;

            fn new<R: AsRef<Self::Storage>, G: AsRef<Self::Storage>, B: AsRef<Self::Storage>>(
                red: R,
                green: G,
                blue: B,
            ) -> Self {
                <$pixel_type>::new(*red.as_ref(), *green.as_ref(), *blue.as_ref())
            }

            fn red(&self) -> Self::Storage {
                self.r()
            }

            fn green(&self) -> Self::Storage {
                self.g()
            }

            fn blue(&self) -> Self::Storage {
                self.b()
            }
        }
    };
}

impl_pixel_color!(Rgb555, 5, u8);
impl_pixel_color!(Rgb666, 6, u8);
impl_pixel_color!(Rgb888, 8, u8);
