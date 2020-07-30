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
        make_test!(BASIC($test_name)
            jetsam-compile: |_, c| c;
            pyret-compile: |_, c| c;
            => expects: $expected);
    };

    (BASIC($test_name: ident)
    jetsam-compile: $jetsam_compile_override: expr;
    pyret-compile: $pyret_compile_override: expr;
    => expects: $expected: expr) => {
        make_test!(FULL(test => $test_name, data => $test_name)
            jetsam-compile: $jetsam_compile_override;
            pyret-compile: $pyret_compile_override;
            => expects: $expected);
    };

    (FULL(test => $test_name: ident, data => $test_data_name: ident)
    jetsam-compile: $jetsam_compile_override: expr;
    pyret-compile: $pyret_compile_override: expr;
    => expects: $expected: expr) => {

        #[test]
        fn $test_name() {

            use common::{SRC_DIR, BINDING_DIR, ARR_COMPILED_DIR};

            let test_env = common::TestEnv::new(stringify!($test_name));

            let runner = concat!(stringify!($test_data_name), "_runner.arr");

            test_env.create_tmp_dir(SRC_DIR);
            test_env.create_tmp_dir(BINDING_DIR);
            test_env.create_tmp_file(
                src_file!(runner),
                include_str!(concat!("./data/", stringify!($test_data_name), "_runner.arr"))
            );
            test_env.create_tmp_file(
                binding_file!(concat!(stringify!($test_data_name), ".d.ts")),
                include_str!(concat!("./data/", stringify!($test_data_name), ".d.ts"))
            );
            test_env.create_tmp_file(
                binding_file!(concat!(stringify!($test_data_name), ".js")),
                include_str!(concat!("./data/", stringify!($test_data_name), ".js"))
            );

            let mut jetsam_build_cmd = {
                let tmp = test_env
                    .jetsam_build_cmd(binding_file!(concat!(stringify!($test_data_name), ".d.ts")), BINDING_DIR);
                $jetsam_compile_override(&test_env, tmp)
            };
            let jetsam_output = jetsam_build_cmd
                .arg("--ts-flavor")
                .arg("ts-num")
                .output()
                .expect(&format!("jetsam failed (`{:#?}`)", jetsam_build_cmd));

            if !jetsam_output.status.success() {
                let stderr: String = String::from_utf8(jetsam_output.stderr)
                    .expect("Jetsam build did NOT emit utf8 in stderr");

                dbg!(test_env);
                eprintln!("\n=======cmd stderr=======\n\n{}\n\n=======end cmd stderr=======\n", stderr);
                panic!("Command `{:?}` failed with code: {}", jetsam_build_cmd, jetsam_output.status);
            }

            let mut pyret_build_cmd = {
                let tmp = test_env
                    .pyret_build_cmd(runner, SRC_DIR, ARR_COMPILED_DIR);

                $pyret_compile_override(&test_env, tmp)
            };
            let pyret_output = pyret_build_cmd
                .output()
                .expect(&format!("pyret failed [`{:#?}`]", pyret_build_cmd));

            if !pyret_output.status.success() {
                let stderr: String = String::from_utf8(pyret_output.stderr)
                    .expect("Pyret build did NOT emit utf8 in stderr");

                dbg!(test_env);
                eprintln!("\n=======cmd stderr=======\n\n{}\n\n=======end cmd stderr=======\n", stderr);
                panic!("Command `{:?}` failed with code: {}", pyret_build_cmd, pyret_output.status);
            }

            let mut run_pyret_cmd = test_env
                .run_pyret_cmd(py_compiled_file!(project => concat!(stringify!($test_data_name), "_runner.arr.js")));
            let run_output = run_pyret_cmd
                .output()
                .expect("pyret execution failed (i/o error)");

            if !run_output.status.success() {
                let stderr: String = String::from_utf8(run_output.stderr)
                    .expect("Program run (node) did NOT emit utf8 in stderr");

                dbg!(test_env);
                eprintln!("\n=======cmd stderr=======\n\n{}\n\n=======end cmd stderr=======\n", stderr);
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
