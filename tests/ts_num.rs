#[macro_use]
mod macros;
mod common;

// Can get debug result/expected prints by defining env var "DBG_EPRINT"


make_test!(BASIC(basic_ts_num) expects:
    common::line_separated_expected(&["40", "-20", "-55", "99", "9000", "Done"])
);
