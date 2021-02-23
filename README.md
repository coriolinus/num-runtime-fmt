# `num-runtime-fmt`

Format numbers according to runtime specifications.

[![Build and Test Status](https://github.com/coriolinus/num-runtime-fmt/workflows/Build%20and%20Test/badge.svg?branch=main)](https://github.com/coriolinus/num-runtime-fmt/actions?query=branch%3Amain+workflow%3A%22Build+and+Test%22)

> **Note**: This crate is currently under development. This document currently
> serves more as a development guideline than as an accurate description of
> capabilities.

## Why another numeric formatting crate?

This crate implements numeric formatting with a combination of properties not
found elsewhere:

- **Runtime**: The format specifiers do not need to be known in advance
- **Flexible**: The format specification supports nearly a superset of the
  features of the standard library.
- **Focused**: Keeps things simple by offering a very simple interface which can
  format a single number at a time. More complicated formatting jobs can be
  accomplished by passing the output into standard formatting machinery.

### Alternatives

- [`std::fmt`](https://doc.rust-lang.org/std/fmt/) is expressive and built into
  the standard library. However, you must provide the format string at compile
  time, and it can't handle digit separation.
- [`num-format`](https://crates.io/crates/num-format) is fairly comprehensive
  and supports `no-std` applications. However, it doesn't support non-decimal
  output modes or arbitrary groupings.
- [`runtime-fmt`](https://crates.io/crates/runtime-fmt) allows for runtime
  format strings which support everything the standard library does, but it's
  nightly-only.
- [`fomat-macros`](https://crates.io/crates/fomat-macros) provides alternate
  formatting macros with their own syntax, but which appear to be somewhat less
  powerful than those of the stdlib.

## Format String Reference

The formatters in this crate implement a near superset of features available in the
format macros provided by `std::fmt`. However, they are exclusively concerned
with formatting a single numeric value. Therefore, the specification language is
somewhat truncated: it omits both braces and the colon which precedes the format
specification. Therefore, where in the standard formatting machinery you might
write:

```rust
let hex_digit = format!("{:02x}", 0xf0);
```

With this library, the equivalent would be:

```rust
let hex_digit = NumFmt::from_str("02x")?.fmt(0xf0);
```

> **Note**: though these formatters support a superset of features of the
> standard ones, in that anything possible with the standard formatters is
> possible with this library, they do not have a superset of the syntax: while
> the intersection is large, there are a few syntax elements legal in the
> standard formatter which are not legal for this formatter. In particular, the
> standard formatter supports named width parameters. Dynamic width parameters
> are legal in this crate, but they cannot be named.

### Grammar

The gramar for the format string derives substantially from the standard library's:

```text
format_spec := [[fill]align][sign]['#'][['0']width]['.' precision][format][separator[spacing]]
fill := character
align := '<' | '^' | '>' | 'v'
sign := '+' | '-'
width := integer not beginning with '0'
precision := integer
format := 'b' | 'o' | 'd' | 'x' | 'X'
separator := '_', | ',' | ' '
spacing := integer
```

> **Note**: there is no special syntax for dynamic insertion of `with`, `precision` and `spacing`.
> Simply use `NumFmt::format_with`; the dynamic values there always override any values for those
> fields, whether set or unset in the format string.

### `fill`

Any single `char` which precedes an align specifier is construed as the fill
character: when `width` is greater than the actual rendered width of the number,
the excess is padded with this character.

> **Note**: Wide characters are counted according to their bit width, not their
> quantity.

```rust
let heart = '🖤';
assert_eq!(heart.len_utf8(), 4);
let fmt = NumFmt::builder().fill(heart).width(6).build();
// Note that this renders as two characters: we requested a width of 6.
// The number renders as a single character. The heart fills up the next 4 for a total of 5.
// Adding an extra heart would exceed the requested width, so it only renders one.
assert_eq!(fmt.fmt(1), "🖤1");
```

### `align`ment

- `>`: the output is right-aligned in `width` columns (default).
- `^`: the output is centered in `width` columns.
- `<`: the output is left-aligned in `width` columns.
- `v`: attempt to align the decimal point at column index `width`. For integers,
  equivalent to `>`.

### `sign`

- `-`: print a leading `-` for negative numbers, and nothing in particular for
  positive (default)
- `+`: print a leading `+` for positive numbers

### `#`

If a `#` character is present, print a base specification before the number
according to its format (see `format` below).

- binary: `0b`
- octal: `0o`
- decimal: `0d`
- hex: `0x`

This base specification counts toward the width of the number:

```rust
assert_eq!(NumFmt::from_str("#04b").unwrap().fmt(2), "0b10");
```

### `0`

Conceptually, this is similar to the common pattern `0>`; it saves a
char, and looks better when combined with a sign specifier. However, it comes
with a caveat:

```rust
assert_eq!(NumFmt::from_str("-03").unwrap().fmt(-1), "-01");
assert_eq!(NumFmt::from_str("0>-3").unwrap().fmt(-1), "-001");
```

The distinction is that the `0` formatter includes the number's sign in the
desired width; an explicit fill does not include the sign in the width
calculation.

### `width`

This is a parameter for the "minimum width" that the format should take up. If
the value's string does not fill up this many characters, then the padding
specified by fill/alignment will be used to take up the required space (see
`fill` above).

When using the `$` sigil instead of an explicit width, the width can be set
dynamically:

```rust
assert_eq!(NumFmt::from_str("-^$").unwrap().fmt_with(1, Dynamic::width(5)), "--1--");
```

If an explicit width is not provided, defaults to 0.

### `precision`

How many digits after the decimal point are printed. Note that integers can be forced
to emit decimal places with this modifier.

If an explicit precision is not provided, defaults to emitting all post-decimal
digits emitted by the underlying type.

```rust
assert_eq!(NumFmt::from_str(".2").unwrap().fmt(3.14159).unwrap(), "3.14");
assert_eq!(NumFmt::from_str(".7").unwrap().fmt(3.14159).unwrap(), "3.1415900");
```

If the requested precision exceeds the native precision available to this number,
the remainder is always filled with `'0'`, even if `fill` is specified:

```rust
assert_eq!(NumFmt::from_str("-<6.2").unwrap().fmt(1.0_f32).unwrap(), "1.00--");
```

### `format`

- `b`: Emit this number's binary representation
- `o`: Emit this number's octal representation
- `d`: Emit this number's decimal representation (default)
- `x`: Emit this number's hexadecimal representation with lowercase letters
- `X`: Emit this number's hexadecimal representation with uppercase letters

> **Note**: This is one of a few areas where the standard library has
> capabilities this library does not: it supports some other numeric formats.
> Pull requests welcomed to bring this up to parity.

### `separator`

A separator is a (typically non-numeric) character inserted between groups of digits to make
it easier for humans to parse the number when reading. Different separators may
be desirable in different contexts.

- `_`: Separate numeric groups with an underscore
- `,`: Separate numeric groups with a comma
- ` ` (space char): Separate numeric groups with a space

By default, numeric groups are not separated. It is not possible to explicitly
specify that numeric groups are not separated when using a format string.
However, this can be specified when building the formatter via builder.

Wyhen using the builder to explicitly set formatter options, it is also possible
to separate numeric groups with an arbitrary `char`. This can be desirable to
i.e. support German number formats, which use a `.` to separate numeric groups
and a `,` as a decimal separator.

### `spacing`

Spacing determines the number of characters in each character group. It is only
of interest when the separator is set. The default spacing is 3.

Apparently some cultures separate numeric digits with a non-constant group size.
Please file an issue if this feature is important to you.

### Decimal separator

When using the builder to explicitly set formatter options, it is possible to
set the decimal separator to any `char`. This can be desirable to i.e. support
German number formats, which use a `.` to separate numeric groups and a `,` as a
decimal separator.
