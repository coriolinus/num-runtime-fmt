//! Format numbers according to a runtime specification.
//!
//! ## Entry Points
//!
//! The core of this crate is the [`NumFmt`] type. You can [build][NumFmt::builder] it explicitly,
//! or [parse][NumFmt::from_str] it from a format string with similar grammar to that of the
//! standard library.
//!
//! Given an instance of `NumFmt`, you can call its [`fmt`][NumFmt::fmt] method to simply format
//! a number, or its [`fmt_with`][NumFmt::fmt_with] method to apply dynamic parameters.
//!
//! ## Format String Grammar
//!
//! The gramar for the format string derives substantially from the standard library's:
//!
//! ```text
//! format_spec := [[fill]align][sign]['#'][['0']width]['.' precision][format][separator[spacing]]
//! fill := character
//! align := '<' | '^' | '>' | 'v'
//! sign := '+' | '-'
//! width := integer not beginning with '0'
//! precision := integer
//! format := 'b' | 'o' | 'd' | 'x' | 'X'
//! separator := '_', | ',' | ' '
//! spacing := integer
//! ```
//!
//! ### Note
//!
//! There is no special syntax for dynamic insertion of `with`, `precision` and `spacing`.
//! Simply use [`NumFmt::fmt_with`]; the dynamic values provided there always override any
//! values for those fields, whether set or not in the format string.
//!
//! ## `fill`
//!
//! Any single `char` which precedes an align specifier is construed as the fill
//! character: when `width` is greater than the actual rendered width of the number,
//! the excess is padded with this character.
//!
//! ### Note
//! Wide characters are counted according to their quantity, not their bit width.
//!
//! ```rust
//! # use num_runtime_fmt::NumFmt;
//! let heart = '🖤';
//! assert_eq!(heart.len_utf8(), 4);
//! let fmt = NumFmt::builder().fill(heart).width(3).build();
//! let formatted = fmt.fmt(1).unwrap();
//! assert_eq!(formatted, "🖤🖤1");
//! // Note that even though we requested a width of 3, the binary length is 9.
//! assert_eq!(formatted.len(), 9);
//! ```
//!
//! ## `align`ment
//!
//! - `>`: the output is right-aligned in `width` columns (default).
//! - `^`: the output is centered in `width` columns.
//! - `<`: the output is left-aligned in `width` columns.
//! - `v`: attempt to align the decimal point at column index `width`. For integers,
//!   equivalent to `>`.
//!
//! ## `sign`
//!
//! - `-`: print a leading `-` for negative numbers, and nothing in particular for
//!   positive (default)
//! - `+`: print a leading `+` for positive numbers
//!
//! ## `#`
//!
//! If a `#` character is present, print a base specification before the number
//! according to its format (see `format` below).
//!
//! - binary: `0b`
//! - octal: `0o`
//! - decimal: `0d`
//! - hex: `0x`
//!
//! This base specification counts toward the width of the number:
//!
//! ```rust
//! # use num_runtime_fmt::NumFmt;
//! assert_eq!(NumFmt::from_str("#04b").unwrap().fmt(2).unwrap(), "0b10");
//! ```
//!
//! ## `0`
//!
//! Engage the zero handler.
//!
//! The zero handler overrides the padding specification to `0`, and
//! treats pad characters as part of the number, in contrast
//! to the default behavior which treats them as arbitrary spacing.
//!
//! ## Examples
//!
//! ```rust
//! # use num_runtime_fmt::NumFmt;
//! // sign handling
//! assert_eq!(NumFmt::from_str("-03").unwrap().fmt(-1).unwrap(),   "-01");
//! assert_eq!(NumFmt::from_str("0>-3").unwrap().fmt(-1).unwrap(), "-001");
//! ```
//!
//! ```rust
//! # use num_runtime_fmt::NumFmt;
//! // separator handling
//! assert_eq!(NumFmt::from_str("0>7,").unwrap().fmt(1).unwrap(), "0000001");
//! assert_eq!(NumFmt::from_str("07,").unwrap().fmt(1).unwrap(),  "000,001");
//! ```
//!
//! ## `width`
//!
//! This is a parameter for the "minimum width" that the format should take up. If
//! the value's string does not fill up this many characters, then the padding
//! specified by fill/alignment will be used to take up the required space (see
//! `fill` above).
//!
//! When using the `$` sigil instead of an explicit width, the width can be set
//! dynamically:
//!
//! ```rust
//! # use num_runtime_fmt::{NumFmt, Dynamic};
//! assert_eq!(NumFmt::from_str("-^").unwrap().fmt_with(1, Dynamic::width(5)).unwrap(), "--1--");
//! ```
//!
//! If an explicit width is not provided, defaults to 0.
//!
//! ## `precision`
//!
//! Precision will pad or truncate as required if set. If unset, passes through as many
//! digits past the decimal as the underlying type naturally returns.
//!
//! ```rust
//! # use num_runtime_fmt::{NumFmt, Dynamic};
//! assert_eq!(NumFmt::from_str(".2").unwrap().fmt(3.14159).unwrap(), "3.14");
//! assert_eq!(NumFmt::from_str(".7").unwrap().fmt(3.14159).unwrap(), "3.1415900");
//! ```
//!
//! If the requested precision exceeds the native precision available to this number,
//! the remainder is always filled with `'0'`, even if `fill` is specified:
//!
//! ```rust
//! # use num_runtime_fmt::NumFmt;
//! assert_eq!(NumFmt::from_str("-<6.2").unwrap().fmt(1.0_f32).unwrap(), "1.00--");
//! ```
//!
//! ## `format`
//!
//! - `b`: Emit this number's binary representation
//! - `o`: Emit this number's octal representation
//! - `d`: Emit this number's decimal representation (default)
//! - `x`: Emit this number's hexadecimal representation with lowercase letters
//! - `X`: Emit this number's hexadecimal representation with uppercase letters
//!
//! ### Note
//!
//! This is one of a few areas where the standard library has
//! capabilities this library does not: it supports some other numeric formats.
//! Pull requests welcomed to bring this up to parity.
//!
//! ## `separator`
//!
//! A separator is a (typically non-numeric) character inserted between groups of digits to make
//! it easier for humans to parse the number when reading. Different separators may
//! be desirable in different contexts.
//!
//! - `_`: Separate numeric groups with an underscore
//! - `,`: Separate numeric groups with a comma
//! - ` ` (space char): Separate numeric groups with a space
//!
//! By default, numeric groups are not separated. It is not possible to explicitly
//! specify that numeric groups are not separated when using a format string.
//! However, this can be specified when building the formatter via builder.
//!
//! When using the builder to explicitly set formatter options, it is also possible
//! to separate numeric groups with an arbitrary `char`. This can be desirable to
//! i.e. support German number formats, which use a `.` to separate numeric groups
//! and a `,` as a decimal separator.
//!
//! ## `spacing`
//!
//! Spacing determines the number of characters in each character group. It is only
//! of interest when the separator is set. The default spacing is 3.

mod align;
mod base;
mod builder;
mod dynamic;
mod num_fmt;
pub mod numeric_trait;
mod parse;
mod sign;

pub use align::Align;
pub use base::Base;
pub use builder::Builder;
pub use dynamic::Dynamic;
pub use num_fmt::{Error, NumFmt};
pub use numeric_trait::Numeric;
pub use sign::Sign;
