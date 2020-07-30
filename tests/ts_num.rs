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

make_test!(FULL(test => config_unwrapped_ts_num, data => unwrapped_ts_num)
    jetsam-compile: |env: &common::TestEnv, mut c: std::process::Command| {
        env.create_tmp_file(
            "config.json",
            include_str!("./data/unwrapped_ts_num.config1.json")
        );


        c
            .arg("--gen-config")
            .arg(env.get_tmp_path("config.json"));

        c
    };
    pyret-compile: |_, c| c;
    => expects: common::line_separated_expected(&["40", "-20", "-55", "99", "9000", "Done"])
);
