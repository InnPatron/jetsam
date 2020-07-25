//! Inspired by ripgrep (https://github.com/BurntSushi/ripgrep)

use std::error;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};
use std::io::{self, Write};
use std::fs::{self, File};
use std::thread;
use std::time::Duration;

const PYRET_COMPILER_DIR: &'static str = "PYRET_COMPILER_DIR";
const PYRET_RUNTIME_DIR: &'static str = "PYRET_RUNTIME_DIR";
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
}

impl TestEnv {

    pub fn new(name: &str) -> Self {
        let root = env::current_exe()
            .unwrap()
            .parent()
            .expect("Ex dir")
            .to_path_buf();

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
        }
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

    pub fn jetsam_build_cmd<S: AsRef<Path>>(&self, input_path: S, output_dir: S) -> Command {
        let mut cmd = self.jetsam_cmd();

        let input_path = self.tmp_dir.join(input_path);
        let output_dir = self.tmp_dir.join(output_dir);

        cmd
            .stderr(Stdio::inherit())
            .arg("-i")
            .arg(input_path)
            .arg("-o")
            .arg(output_dir);

        cmd
    }

    pub fn pyret_cmd(&self) -> Command {
        todo!();
    }

    pub fn pyret_build_cmd(&self) -> Command {
        todo!();
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