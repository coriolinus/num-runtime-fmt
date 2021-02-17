//! This module contains implementations of [`Numeric`] for several types, plus helpers which can
//! ease implementation for your own type.

use std::ops::{BitAnd, ShrAssign};

macro_rules! impl_iter {
    ($iter:ident) => {
        impl<N> $iter<N> {
            pub fn new(n: N) -> Self {
                $iter(n)
            }
        }

        impl<N> Iterator for $iter<N>
        where
            N: Clone + From<u8> + BitAnd<Output = N> + ShrAssign + PartialEq,
        {
            type Item = char;

            fn next(&mut self) -> Option<Self::Item> {
                if self.0 == 0.into() {
                    return None;
                }
                let digit = self.0.clone() & Self::MASK.into();
                self.0 >>= Self::WIDTH.into();
                // this isn't an _efficient_ approach, but it avoids needing a bound to convert
                // from N to u8, which won't always be implemented for interesting types.
                //
                // A custom runtime formatting library can be excused a few inefficiencies.
                for maybe_digit in 0..=Self::MASK {
                    if digit == maybe_digit.into() {
                        if maybe_digit < 10 {
                            return Some((maybe_digit + b'0') as char);
                        } else {
                            return Some((maybe_digit - 10 + b'a') as char);
                        }
                    }
                }
                panic!(
                    "no digit matched when computing {}",
                    std::any::type_name::<$iter<N>>()
                );
            }
        }
    };
}

/// Iterator over binary digits of `N`.
pub struct BinIter<N>(N);

impl<N> BinIter<N> {
    const WIDTH: u8 = 1;
    const MASK: u8 = 1;
}

impl_iter!(BinIter);

/// Iterator over octal digits of `N`.
pub struct OctIter<N>(N);

impl<N> OctIter<N> {
    const WIDTH: u8 = 3;
    const MASK: u8 = 0b0111;
}

impl_iter!(OctIter);

/// Iterator over hexadecimal digits of `N`.
pub struct HexIter<N>(N);

impl<N> HexIter<N> {
    const WIDTH: u8 = 4;
    const MASK: u8 = 0b1111;
}

impl_iter!(HexIter);

#[cfg(test)]
mod tests {
    macro_rules! suite_for {
        ($iter:ident, $fmt:literal, $( $test_type:ident ),+) => {
            #[allow(non_snake_case)]
            mod $iter {
                use super::super::$iter as Iter;
                use std::convert::TryInto;

                $(
                    #[test]
                    fn $test_type() {
                        // don't check 0: that needs special casing for all these impls
                        for n in 1..=$test_type::MAX.min(1024.try_into().unwrap_or($test_type::MAX)) {
                            let expect = format!($fmt, n);
                            let mut digits: Vec<_> = Iter::new(n).map(|d| d.to_string()).collect();
                            digits.reverse();
                            let actual = digits.join("");
                            dbg!(&actual, &expect);
                            assert_eq!(actual, expect);
                        }
                    }
                )+
            }
        };
    }

    // we don't test `i8` because it doesn't implement From<u8>.
    // however, circumstantial evidence suggests that the implementation would probably work fine.

    suite_for!(BinIter, "{:b}", u8, u16, u32, u64, u128, usize, i16, i32, i64, i128, isize);
    suite_for!(OctIter, "{:o}", u8, u16, u32, u64, u128, usize, i16, i32, i64, i128, isize);
    suite_for!(HexIter, "{:x}", u8, u16, u32, u64, u128, usize, i16, i32, i64, i128, isize);
}
