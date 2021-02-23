#[allow(unused)]
use num_runtime_fmt::{Dynamic, Error, NumFmt, Numeric};

macro_rules! test_mod {
    // module name
    ($mod_name:ident : $( $rest:tt )* ) => {
        mod $mod_name {
            use super::*;

            test_mod!(funcs $( $rest )*);
        }
    };
    // non-dynamic test format error
    (funcs fmt_fail $test_name:ident($fmt:expr, $num:expr, $want:pat); $( $rest:tt )*) => {
        #[test]
        fn $test_name() {
            let fmt = NumFmt::from_str($fmt).expect("must parse expected format");
            dbg!(&fmt);
            let result = fmt.fmt($num).unwrap_err();
            assert!(matches!(result, $want));
        }

        test_mod!(funcs $( $rest )* );
    };
    // non-dynamic
    (funcs $test_name:ident($fmt:expr, $num:expr, $want:expr); $( $rest:tt )*) => {
        #[test]
        fn $test_name() {
            let fmt = NumFmt::from_str($fmt).expect("must parse expected format");
            dbg!(&fmt);
            let result = fmt.fmt($num).expect("must format without error");
            assert_eq!(result, $want);
        }

        test_mod!(funcs $( $rest )* );
    };
    // dynamic
    (funcs $test_name:ident($fmt:expr, $dynamic:expr, $num:expr, $want:expr); $( $rest:tt )*) => {
        #[test]
        fn $test_name() {
            let fmt = NumFmt::from_str($fmt).expect("must parse expected format");
            dbg!(&fmt);
            let result = fmt
                .fmt_with($num, $dynamic)
                .expect("must format without error");
            assert_eq!(result, $want);
        }

        test_mod!(funcs $( $rest )* );
    };
    // base case to terminate recursion
    (funcs) => {};
}

test_mod! { fill:
    left_int(":<5", 1, "1::::");
    middle_int(":^5", 1, "::1::");
    middle_offset_int(":^4", 1, "::1:");
    right_int(":>5", 1, "::::1");
    decimal_int(":v5", 1, "::::1");

    left_float(":<5", 1.1, "1.1::");
    middle_float(":^5", 1.1, ":1.1:");
    middle_offset_float(":^6", 1.1, "::1.1:");
    right_float(":>5", 1.1, "::1.1");
    decimal_float(":v5", 1.1, "::::1.1");
    bigger_decimal_float(":v5", 11.1, ":::11.1");
}

test_mod! { align:
    left_int("<5", 1, "1    ");
    middle_int("^5", 1, "  1  ");
    middle_offset_int("^4", 1, "  1 ");
    right_int(">5", 1, "    1");
    decimal_int("v5", 1, "    1");

    left_float("<5", 1.1, "1.1  ");
    middle_float("^5", 1.1, " 1.1 ");
    middle_offset_float("^6", 1.1, "  1.1 ");
    right_float(">5", 1.1, "  1.1");
    decimal_float("v5", 1.1, "    1.1");
    bigger_decimal_float("v5", 11.1, "   11.1");
}

test_mod! { sign:
    omit_pos_int("",   1, "1");
    omit_neg_int("",  -1, "-1");
    plus_pos_int("+",  1, "+1");
    plus_neg_int("+", -1, "-1");
    mins_pos_int("-",  1, "1");
    mins_neg_int("-", -1, "-1");

    omit_pos_float("",   1.1, "1.1");
    omit_neg_float("",  -1.1, "-1.1");
    plus_pos_float("+",  1.1, "+1.1");
    plus_neg_float("+", -1.1, "-1.1");
    mins_pos_float("-",  1.1, "1.1");
    mins_neg_float("-", -1.1, "-1.1");
}

test_mod! { hash:
    binary("#b", 15, "0b1111");
    octal("#o", 15, "0o17");
    decimal_implied("#", 15, "0d15");
    decimal_explicit("#d", 15, "0d15");
    hex_lower("#x", 15, "0xf");
    hex_upper("#X", 15, "0xF");
    decimal_float_implied("#", 1.1, "0d1.1");
    decimal_float_explicit("#d", 1.1, "0d1.1");
}

test_mod! { zero:
    unused_int("01", 1, "1");
    unnused_float("01", 1.1, "1.1");

    bare_pos_int("05", 1, "00001");
    bare_neg_int("05", -1, "-0001");
    bare_pos_float("05", 1.1, "001.1");
    bare_neg_float("05", -1.1, "-01.1");

    right_pos_int(">05", 1, "00001");
    right_neg_int(">05", -1, "-0001");
    right_pos_float(">05", 1.1, "001.1");
    right_neg_float(">05", -1.1, "-01.1");

    dec_pos_int("v05", 1, "00001");
    dec_neg_int("v05", -1, "-0001");
    dec_pos_float("v05", 1.1, "00001.1");
    dec_neg_float("v05", -1.1, "-0001.1");

    fmt_fail center_pos_int("^05", 1, Error::IncompatibleAlignment);
    fmt_fail center_neg_int("^05", -1, Error::IncompatibleAlignment);
    fmt_fail center_pos_float("^05", 1.1, Error::IncompatibleAlignment);
    fmt_fail center_neg_float("^05", -1.1, Error::IncompatibleAlignment);

    fmt_fail left_pos_int("<05", 1, Error::IncompatibleAlignment);
    fmt_fail left_neg_int("<05", -1, Error::IncompatibleAlignment);
    fmt_fail left_pos_float("<05", 1.1, Error::IncompatibleAlignment);
    fmt_fail left_neg_float("<05", -1.1, Error::IncompatibleAlignment);
}

test_mod! { width:
    narrow_noop_pos_int("1", 123, "123");
    narrow_noop_neg_int("1", -123, "-123");
    narrow_noop_pos_float("1", 1.3, "1.3");
    narrow_noop_neg_float("1", -1.3, "-1.3");

    default_align_pos_int("5", 1, "    1");
    default_align_neg_int("5", -1, "   -1");
    default_align_pos_float("5", 1.1, "  1.1");
    default_align_neg_float("5", -1.1, " -1.1");

    dyn_default_align_pos_int("", Dynamic::width(5), 1, "    1");
    dyn_default_align_neg_int("", Dynamic::width(5), -1, "   -1");
    dyn_default_align_pos_float("", Dynamic::width(5), 1.1, "  1.1");
    dyn_default_align_neg_float("", Dynamic::width(5), -1.1, " -1.1");
}

test_mod! { precision:
    truncate(".2", 3.14159, "3.14");
    extend(".7", 3.14159, "3.1415900");

    with_width("5.3", 1.2, "1.200");
    width_extend("7.3", 1.2, "  1.200");
    fill_extend("|^7.3", 1.2, "|1.200|");
    decimal("v7.3", 1.2, "      1.200");
    zero_dec("v05.3", 1.2, "00001.200");
    left("<7.3", 1.2, "1.200  ");
}

test_mod! { base:
    binary("09b_4", 0b1101, "0000_1101");
    octal("04o", 0o644, "0644");
    lower_hex("x", 0xcafebabe_u32, "cafebabe");
    upper_hex("X", 0xDEADBEEF_u32, "DEADBEEF");

    fmt_fail binary_float("09b_4", 0.0, Error::NotImplemented(_, _));
    fmt_fail octal_float("04o", 0.0, Error::NotImplemented(_, _));
    fmt_fail lower_hex_float("x", 0.0, Error::NotImplemented(_, _));
    fmt_fail upper_hex_float("X", 0.0, Error::NotImplemented(_, _));
}
