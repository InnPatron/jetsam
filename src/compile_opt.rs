use std::path::PathBuf;

use crate::ts::TsFlavor;
use crate::generate::{EmitConfig, GenConfig};

pub struct CompileOpt<'a> {
    pub input_path: PathBuf,
    pub require_path: Option<&'a str>,
    pub file_stem: Option<&'a str>,
    pub output_dir: PathBuf,

    pub ts_flavor: TsFlavor,
    pub gen_config: GenConfig,
    pub emit_config: EmitConfig,
}
