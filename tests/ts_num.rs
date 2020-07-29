#[macro_use]
mod macros;
mod common;

// Can get debug result/expected prints by defining env var "DBG_EPRINT"


make_test!(BASIC(basic_ts_num) expects:
    common::line_separated_expected(&["40", "-20", "-55", "99", "9000", "Done"])
);

make_test!(BASIC(unwrapped_ts_num)
    jetsam-compile: |_, mut c: std::process::Command| {
        c
            .arg("--wrap-top-level-vars")
            .arg("false");

        c
    };
    pyret-compile: |_, c| c;
    => expects: common::line_separated_expected(&["40", "-20", "-55", "99", "9000", "Done"])
);
