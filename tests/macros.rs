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
    };

    (project => $f: expr) => {
        format!("{}/project/{}", ARR_COMPILED_DIR, $f)
    }
}

/// Can get debug result/expected prints by defining env var "DBG_EPRINT"
macro_rules! make_test {
    (BASIC($test_name: ident) expects: $expected: expr) => {

        #[test]
        fn $test_name() {

            use common::{SRC_DIR, BINDING_DIR, ARR_COMPILED_DIR};

            let test_env = common::TestEnv::new(stringify!($test_name));

            let runner = concat!(stringify!($test_name), "_runner.arr");

            test_env.create_tmp_dir(SRC_DIR);
            test_env.create_tmp_dir(BINDING_DIR);
            test_env.create_tmp_file(
                src_file!(runner),
                include_str!(concat!("./data/", stringify!($test_name), "_runner.arr"))
            );
            test_env.create_tmp_file(
                binding_file!(concat!(stringify!($test_name), ".d.ts")),
                include_str!(concat!("./data/", stringify!($test_name), ".d.ts"))
            );
            test_env.create_tmp_file(
                binding_file!(concat!(stringify!($test_name), ".js")),
                include_str!(concat!("./data/", stringify!($test_name), ".js"))
            );

            let mut jetsam_build_cmd = test_env
                .jetsam_build_cmd(binding_file!(concat!(stringify!($test_name), ".d.ts")), BINDING_DIR);
            let jetsam_output = jetsam_build_cmd
                .arg("--ts-flavor")
                .arg("ts-num")
                .output()
                .expect("jetsam failed (i/o error)");

            if !jetsam_output.status.success() {
                dbg!(test_env);
                panic!("Command `{:?}` failed with code: {}", jetsam_build_cmd, jetsam_output.status);
            }

            let mut pyret_build_cmd = test_env
                .pyret_build_cmd(runner, SRC_DIR, ARR_COMPILED_DIR);
            let pyret_output = pyret_build_cmd
                .output()
                .expect("pyret failed (i/o error)");

            if !pyret_output.status.success() {
                dbg!(test_env);
                panic!("Command `{:?}` failed with code: {}", pyret_build_cmd, pyret_output.status);
            }

            let mut run_pyret_cmd = test_env
                .run_pyret_cmd(py_compiled_file!(project => concat!(stringify!($test_name), "_runner.arr.js")));
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

            let expected = $expected;

            if expected != run_stdout {
                use std::env;

                if std::env::var_os("DBG_EPRINT").is_some() {
                    eprintln!("Expected:\n{:#?}", expected);
                    eprintln!("Result:\n{:#?}", run_stdout);
                } else {
                    eprintln!("Expected:\n{}", expected);
                    eprintln!("Result:\n{}", run_stdout);
                }
                panic!("Expected did not equal result");
            }
        }
    }
}
