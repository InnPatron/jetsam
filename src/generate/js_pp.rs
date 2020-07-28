use std::io::{self, Write};

use swc_ecma_ast::*;

pub struct PrettyPrinter<W: Write> {
    writer: W,
    indent: u64,
}

impl<W: Write> PrettyPrinter<W> {
    pub fn print(writer: W, module: &Module) -> io::Result<()> {
        todo!();
    }
}
