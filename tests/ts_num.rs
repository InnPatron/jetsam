#[macro_use]
mod macros;
mod common;

const BINDING_DIR: &'static str = "src/bindings";

macro_rules! binding_file {
    ($f: expr) => {
        format!("{}/{}", BINDING_DIR, $f)
    }
}

#[test]
fn i_test() {
    common::check_aux_bins().unwrap();

    let test_env = common::TestEnv::new("i_test");

    test_env.create_tmp_dir("src");
    test_env.create_tmp_dir("src/bindings");
    test_env.create_tmp_dir(BINDING_DIR);
    test_env.create_tmp_file(binding_file!("basic_ts_num.d.ts"), include_str!("./data/basic_ts_num.d.ts"));
    test_env.create_tmp_file(binding_file!("basic_ts_num.js"), include_str!("./data/basic_ts_num.js"));


    let mut build_cmd = test_env.jetsam_build_cmd(binding_file!("basic_ts_num.d.ts"), BINDING_DIR);
    let output = build_cmd
        .arg("--ts-flavor")
        .arg("ts-num")
        .output()
        .expect("jetsam failed (i/o error)");

    if !output.status.success() {
        dbg!(test_env);
        return panic!("Command `{:?}` failed with code: {}", build_cmd, output.status);
    }
}
