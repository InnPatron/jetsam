mod error;
mod bind_gen;
mod structures;

use std::sync::Arc;
use std::path::PathBuf;

use swc_common::{
    errors::{ColorConfig, Handler},
    FileName, FilePathMapping, SourceMap,
};
use swc_ecma_parser::{lexer::Lexer, Parser, Session, SourceFileInput, Syntax, TsConfig, JscTarget};

fn main() {
    swc_common::GLOBALS.set(&swc_common::Globals::new(), || {
        let cm: Arc<SourceMap> = Default::default();
        let handler =
            Handler::with_tty_emitter(ColorConfig::Auto, true, false,
Some(cm.clone()));

        let path = PathBuf::from("../ts-tests/src/type_alias.d.ts");
        let context =
            bind_gen::Context::new(
                path,
                &handler,
                cm.clone(),
            );
        let module = bind_gen::open_module(&context, None)
            .expect("Failed to open module");

        let _ = bind_gen::process_module(context, module)
            .expect("Bind gen failure");
    });
}
