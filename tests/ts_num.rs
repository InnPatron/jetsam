#[macro_use]
mod macros;
mod common;

#[test]
fn i_test() {
    common::check_aux_bins().unwrap();

    let test_env = common::TestEnv::new("i_test");

    test_env.create_tmp_dir("src");
    test_env.create_tmp_dir("build");
    test_env.create_tmp_file("src/basic_ts_num.d.ts", include_str!("./data/basic_ts_num.d.ts"));


    let mut build_cmd = test_env.jetsam_build_cmd("src/basic_ts_num.d.ts", "build");
    let output = build_cmd
        .arg("--ts-flavor")
        .arg("ts-num")
        .output()
        .expect("jetsam failed (i/o error)");

    if !output.status.success() {
        return panic!("Command `{:?}` failed with code: {}", build_cmd, output.status);
    }
}
