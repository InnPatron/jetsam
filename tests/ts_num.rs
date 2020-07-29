#[macro_use]
mod macros;
mod common;

use common::{SRC_DIR, BINDING_DIR, ARR_COMPILED_DIR};

macro_rules! binding_file {
    ($f: expr) => {
        format!("{}/{}", BINDING_DIR, $f)
    }
}

macro_rules! src_file {
    ($f: expr) => {
        format!("{}/{}", SRC_DIR, $f)
    }
}

macro_rules! py_compiled_file {
    ($f: expr) => {
        format!("{}/{}", ARR_COMPILED_DIR, $f)
    }
}

#[test]
fn basic_ts_num_runner() {
    common::check_aux_bins().unwrap();

    let test_env = common::TestEnv::new("basic_ts_num_runner");

    test_env.create_tmp_dir(SRC_DIR);
    test_env.create_tmp_dir(BINDING_DIR);
    test_env.create_tmp_file(src_file!("basic_ts_num_runner.arr"), include_str!("./data/basic_ts_num_runner.arr"));
    test_env.create_tmp_file(binding_file!("basic_ts_num.d.ts"), include_str!("./data/basic_ts_num.d.ts"));
    test_env.create_tmp_file(binding_file!("basic_ts_num.js"), include_str!("./data/basic_ts_num.js"));


    let mut jetsam_build_cmd = test_env.jetsam_build_cmd(binding_file!("basic_ts_num.d.ts"), BINDING_DIR);
    let jetsam_output = jetsam_build_cmd
        .arg("--ts-flavor")
        .arg("ts-num")
        .output()
        .expect("jetsam failed (i/o error)");

    if !jetsam_output.status.success() {
        dbg!(test_env);
        panic!("Command `{:?}` failed with code: {}", jetsam_build_cmd, jetsam_output.status);
    }

    let mut pyret_build_cmd = test_env.pyret_build_cmd("basic_ts_num_runner.arr", SRC_DIR, ARR_COMPILED_DIR);
    let pyret_output = pyret_build_cmd
        .output()
        .expect("pyret failed (i/o error)");

    if !pyret_output.status.success() {
        dbg!(test_env);
        panic!("Command `{:?}` failed with code: {}", pyret_build_cmd, pyret_output.status);
    }

    let mut run_pyret_cmd = test_env.run_pyret_cmd(py_compiled_file!("project/basic_ts_num_runner.arr.js"));
    let run_output = run_pyret_cmd
        //.stdout(std::process::Stdio::inherit())
        .output()
        .expect("pyret execution failed (i/o error)");

    if !run_output.status.success() {
        dbg!(test_env);
        panic!("Command `{:?}` failed with code: {}", run_pyret_cmd, run_output.status);
    }

    let run_stdout: String = String::from_utf8(run_output.stdout)
        .expect("Pyret execution did NOT emit utf8 in stdout");

    let expected = common::line_separated_expected(&["40", "-20", "-55", "99", "Done"]);

    assert_eq!(expected, run_stdout);
}
