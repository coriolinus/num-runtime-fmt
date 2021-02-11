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

## Format Reference

The formatters in this crate implement a superset of features available in the
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
let hex_digit = NumFmt::from_str("02x")?.format(0xf0);
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
format_spec := [[fill]align][sign]['#']['0'][width]['.' precision][type]
fill := character
align := '<' | '^' | '>' | 'v'
sign := '+' | '-'
format := '#'
width := count
precision := count
type := 'b' | 'o' | 'd' | 'x' | 'X'
count := '$' | integer
```

### `fill`

Any single `char` which precedes an align specifier is construed as the fill
character: when `width` is greater than the actual rendered width of the number,
the excess is padded with the ASCII character corresponding to this byte.

> **Note**: Wide characters are counted according to their bit width, not their
> quantity.

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

If a `#` character is present, print a base specification before the number (see
below):

- binary: `0b`
- octal: `0o`
- decimal: `0d`
- hex: `0x`

### `0`

Conceptually, this is shorthand for the common pattern `0>`; it just saves a
char, and looks better when combined with a sign specifier. However, it comes
with a caveat:

```rust
assert_eq!(NumFmt::from_str("-03").unwrap().format(-1), "-01");
assert_eq!(NumFmt::from_str("0>-3").unwrap().format(-1), "-001");
```

The distinction is that the `0` formatter includes the number's sign in the
desired width; an explicit fill does not include the sign in the width
calculation.

### Width

This is a parameter for the "minimum width" that the format should take up. If
the value's string does not fill up this many characters, then the padding
specified by fill/alignment will be used to take up the required space (see
below).

When using the `$` sigil instead of an explicit width, the width can be set
dynamically:

```rust
assert_eq!(NumFmt::from_str("-^$").unwrap().format_with(1, Dynamic::width(5)), "--1--");
```

If an explicit width is not provided, defaults to 0.

### Precision

Ignored for integers.

For non-integers, this is how many digits after the decimal point are printed.

```rust
assert_eq!(NumFmt::from_str("|^.$").unwrap().format_with(1, Dynamic::precision(5)), "|0.3|");
```

If an explicit precision is not provided, defaults to 0.

### Type

- `b`: Emit this number's binary representation
- `o`: Emit this number's octal representation
- `d`: Emit this number's decimal representation (default)
- `x`: Emit this number's hexadecimal representation with lowercase letters
- `X`: Emit this number's hexadecimal representation with uppercase letters

> **Note**: This is one of a few areas where the standard library has
> capabilities this library does not: it supports some other numeric formats.
> Pull requests welcomed to bring this up to parity.

### TODO

- group separator char
- group width

### Other Options

These options require explicitly building a `NumFmt` instance, but allow control
of options not available via the format strings.

- decimal separator
- non-constant group widths (i.e. Indian)
