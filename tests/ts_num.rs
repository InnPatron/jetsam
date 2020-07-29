#[macro_use]
mod macros;
mod common;

make_test!(BASIC(basic_ts_num) expects:
    common::line_separated_expected(&["40", "-20", "-55", "99", "Done"])
);
