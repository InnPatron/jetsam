//! Inspired by ripgrep (https://github.com/BurntSushi/ripgrep)

use std::env;
use std::path::{Path, PathBuf};
use std::process::{self, Command};


#[derive(Debug, Clone)]
pub struct TestEnv {
    root: PathBuf,
    output_dir: PathBuf,
}
