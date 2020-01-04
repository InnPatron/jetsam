mod error;
mod bind_gen;
mod structures;

use std::sync::Arc;
use std::path::Path;

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

        let session = Session { handler: &handler };

        // Real usage
        let fm = cm
             .load_file(Path::new("../three.js/src/utils.d.ts"))
             .expect("failed to load test");

        let lexer = Lexer::new(
            session,
            Syntax::Typescript(TsConfig {
                tsx: false,
                decorators: false,
                dynamic_import: false,
            }),
            JscTarget::Es2018,
            SourceFileInput::from(&*fm),
            None,
        );

        let mut parser = Parser::new_from(session, lexer);

        let _module = parser
            .parse_module()
            .map_err(|mut e| {
                e.emit();
                ()
            })
            .expect("failed to parser module");

        dbg!(&_module);
    });
}
