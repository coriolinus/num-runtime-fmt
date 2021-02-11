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
- **Flexible**: The format specification is a superset of the format specifiers
  in the standard library.
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
write

```rust
let hex_digit = format!("{:02x}", 0xf0);
```

With this library, the equivalent would be

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

Subsequent sections of this reference are arranged the their positional order in
the format string. All are optional.

### Sign

- `-`: print a leading `-` for negative numbers, and nothing in particular for
  positive (default)
- `+`: print a leading `+` for positive numbers

### Format

If a `#` character is present, print a base specification before the number (see
below):

- binary: `0b`
- octal: `0o`
- decimal: `0d`
- hex: `0x`

### Fill

Any single `char` except an alignment specifier (see below), sign specifier (see
above), or format specifier (see above), which precedes a width specifier is
construed as the fill character: when `width` is greater than the actual
rendered width of the number, the excess is padded with the ASCII character
corresponding to this byte.

> **Note**: Wide characters are counted by their bit width, not their quantity.

### Alignment

- `>`: the output is right-aligned in `width` columns (default)
- `^`: the output is centered in `width` columns.
- `<`: the output is left-aligned in `width` columns
- `v`: attempt to align the decimal point at column index `width`. For integers,
  equivalent to `>`.

### Width

- `N`: use `N` as the desired width.
- `$`: use a dynamic width from the `format_with` method.

This is a parameter for the "minimum width" that the format should take up. If
the value's string does not fill up this many characters, then the padding
specified by fill/alignment will be used to take up the required space (see
below).

The value for the width can also be set dynamically: instead of an integer, use
a `$` sigil and the alternate function:

```rust
assert_eq!(NumFmt::from_str("-^$").unwrap().format_with(1, Dynamic::width(5)), "--1--");
```

If an explicit width is not provided, defaults to 0.

### Precision

- `.N`: use `N` as the desired precision.
- `.$`: use a dynamic precision from the `format_with` method.

Ignored for integers.

For non-integers, this is how many digits after the decimal point are printed.

```rust
assert_eq!(NumFmt::from_str("|^.$").unwrap().format_with(1, Dynamic::precision(5)), "|0.3|");
```

If an explicit precision is not provided, defaults to 0.

### TODO

- group separator char
- group width

### Other Options

These options require explicitly building a `NumFmt` instance, but allow control
of options not available via the format strings.

- decimal separator
- non-constant group widths (i.e. Indian)
