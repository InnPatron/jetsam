//! Inspired by ripgrep (https://github.com/BurntSushi/ripgrep)
// The MIT License (MIT)
//
// Copyright (c) 2015 Andrew Gallant
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

use std::error;
use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::io::{self, Write};
use std::fs::{self, File};
use std::thread;
use std::time::Duration;

pub const SRC_DIR: &'static str = "src";
pub const BINDING_DIR: &'static str = "src/bindings";
pub const ARR_COMPILED_DIR: &'static str = "compiled";

const PYRET_COMPILER_DIR: &'static str = "PYRET_COMPILER_DIR";
const PYRET_RUNTIME_DIR: &'static str = "PYRET_RUNTIME_DIR";
const NODE_PATH: &'static str = "NODE_PATH";
const PYRET_COMPILER_NAME: &'static str = "pyret.jarr";

const TEST_DIR: &'static str = "jetsam-tests";

#[derive(Debug, Clone)]
pub struct TestEnv {

    root: PathBuf,

    /// Determined by environment variable "TMPDIR"
    tmp_dir: PathBuf,

    /// Determined by environment variable contained by constant PYRET_COMPILER_DIR
    pyret_compiler_path: PathBuf,

    /// Determined by environment variable contained by constant PYRET_RUNTIME_DIR.
    /// If that variable is not set, calculate it from the variable denoted by constant PYRET_COMPILER_DIR
    ///     by appending "../runtime"
    pyret_runtime_dir: PathBuf,

    /// By default, assumes `node` is in PATH
    ///   Otherwise, point to node binary with NODE_PATH
    node_path: PathBuf,
}

impl TestEnv {

    pub fn new(name: &str) -> Self {
        let root = env::current_exe()
            .unwrap()
            .parent()
            .expect("Ex dir")
            .to_path_buf();

        let node_path = env::var_os(NODE_PATH)
            .map(PathBuf::from)
            .unwrap_or(PathBuf::from("node"));

        let tmp_dir = {
            let tmp_dir =
                env::temp_dir().join(TEST_DIR).join(name);

                if tmp_dir.exists() {
                    nice_err(&tmp_dir, fs::remove_dir_all(&tmp_dir));
                }
                nice_err(&tmp_dir, repeat(|| fs::create_dir_all(&tmp_dir)));
            tmp_dir
        };

        let pyret_compiler_path = {
            let mut pyret_compiler_path = PathBuf::from(
                env::var_os(PYRET_COMPILER_DIR)
                    .expect(&format!("Missing Pyret compiler path ({} not set)", PYRET_COMPILER_DIR))
            );

            pyret_compiler_path.push(PYRET_COMPILER_NAME);

            if !pyret_compiler_path.exists() {
                panic!("Path to Pyret compiler does not exist ('{}')", pyret_compiler_path.display());
            }

            pyret_compiler_path
        };

        let pyret_runtime_dir = {
            let mut pyret_runtime_dir = env::var_os(PYRET_RUNTIME_DIR)
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    let mut compiler_dir = PathBuf::from(
                        env::var_os(PYRET_COMPILER_DIR)
                        .expect(&format!("Missing Pyret runtime path ({} or {} not set)",
                            PYRET_COMPILER_DIR, PYRET_RUNTIME_DIR))
                    );
                    compiler_dir.pop();
                    compiler_dir.push("runtime");

                    compiler_dir
                });

            if !pyret_runtime_dir.exists() {
                panic!("Path to Pyret runtime does not exist ('{}')", pyret_runtime_dir.display());
            }

            pyret_runtime_dir
        };

        TestEnv {
            root,
            tmp_dir,
            pyret_compiler_path,
            pyret_runtime_dir,
            node_path,
        }
    }

    pub fn get_tmp_path<P: AsRef<Path>>(&self, p: P) -> PathBuf {
        self.tmp_dir.join(p)
    }

    pub fn create_tmp_dir<P: AsRef<Path>>(&self, path: P) {
        let path = self.tmp_dir.join(path);
        nice_err(&path, repeat(|| fs::create_dir_all(&path)));
    }

    pub fn create_tmp_file<P: AsRef<Path>>(&self, name: P, contents: &str) {
        let path = self.tmp_dir.join(&name);
        nice_err(&path, (||{
            let path = self.tmp_dir.join(name);
            let mut file = File::create(path)?;
            file.write_all(contents.as_bytes())?;
            file.flush()
        })());
    }

    pub fn jetsam_cmd(&self) -> Command {
        let jetsam = self.root.join(format!("../jetsam{}", env::consts::EXE_SUFFIX));

        Command::new(jetsam)
    }

    pub fn jetsam_build_cmd<S1: AsRef<Path>, S2: AsRef<Path>>(&self, input_path: S1, output_dir: S2) -> Command {
        let mut cmd = self.jetsam_cmd();

        let input_path = self.tmp_dir.join(input_path);
        let output_dir = self.tmp_dir.join(output_dir);

        cmd
            .arg("-i")
            .arg(input_path)
            .arg("-o")
            .arg(output_dir);

        cmd
    }

    pub fn pyret_cmd(&self) -> Command {
        let mut pyret = Command::new(&self.node_path);
        pyret
            .arg(self.pyret_compiler_path.as_path());

        pyret
    }

    pub fn pyret_build_cmd<S1: AsRef<OsStr>, S2: AsRef<Path>, S3: AsRef<Path>>(&self, root_arr_file: S1, base_path: S2, compiled_path: S3) -> Command {
	// node $(ANCHOR_COMPILER) --type-check true --builtin-js-dir $(ANCHOR_RUNTIME)  --build-runnable $(MAIN)
        let mut pyret = self.pyret_cmd();

        pyret

            .arg("--compiled-dir")
            .arg(self.tmp_dir.join(compiled_path))

            .arg("--base-dir")
            .arg(self.tmp_dir.join(base_path))

            .arg("--type-check")
            .arg("true")

            .arg("--builtin-js-dir")
            .arg(self.pyret_runtime_dir.as_path())

            .arg("--build-runnable")
            .arg(root_arr_file);

        pyret
    }

    pub fn run_pyret_cmd<S1: AsRef<Path>>(&self, module: S1) -> Command {
        let mut pyret = Command::new(&self.node_path);

        pyret
            .arg(self.tmp_dir.join(module));

        pyret
    }
}

pub fn check_aux_bins() -> Result<(), String> {
    let node = Command::new("node")
        .arg("--version")
        .output()
        .map_err(|e| format!("Error finding `node`: {:?}", e))?;

    if !node.status.success() {
        return Err(format!("`node` command failed with code: {}", node.status));
    }

    Ok(())
}

fn nice_err<T, E: error::Error>(path: &Path, res: Result<T, E>) -> T {
    match res {
        Ok(t) => t,
        Err(err) => panic!("{}: {:?}", path.display(), err),
    }
}

fn repeat<F: FnMut() -> io::Result<()>>(mut f: F) -> io::Result<()> {
    let mut last_err = None;
    for _ in 0..10 {
        if let Err(err) = f() {
            last_err = Some(err);
            thread::sleep(Duration::from_millis(500));
        } else {
            return Ok(());
        }
    }
    Err(last_err.unwrap())
}

pub fn line_separated_expected<T: IntoIterator<Item=I>, I: std::fmt::Display>(iter: T) -> String {
    let mut output = String::new();

    for t in iter.into_iter() {
        std::fmt::write(&mut output, format_args!("{}\n", t)).unwrap();
    }

    output.push_str("All tests pass\n");

    output
}
