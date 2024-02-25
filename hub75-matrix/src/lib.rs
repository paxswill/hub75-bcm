#![no_std]

pub mod buffer;
mod clock_divider;
pub mod color;
pub mod config;
pub mod dma;
pub mod rgb_matrix;
mod util;

#[macro_export]
macro_rules! const_check {
    ($const:expr, $test:expr, $msg:literal) => {{
        if $test {
            panic!($msg)
        } else {
            $const
        }
    }};
}

#[macro_export]
macro_rules! const_not_zero {
    ($id:ident, $ty:ty) => {
        const $id: $ty = crate::const_check!($id, $id > 0, "$id cannot be 0");
    };
}
