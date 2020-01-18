use std::path::PathBuf;

// TODO: Add way to change file extension
pub fn prepare_path(path: &mut PathBuf) {
    if path.file_name().is_none() {
        panic!("Module path must contain a file");
    }

    if path.extension().is_none() {
        path.set_extension("d.ts");
    }
}
