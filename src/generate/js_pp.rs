use std::io::{self, Write};

use swc_ecma_ast::*;
use swc_ecma_visit::{Node, Visit};

macro_rules! indent {
    ($self: expr => $b: tt) => {
        $self.indent_right();
        $b
        $self.indent_left();
    }
}

macro_rules! no_err {
    ($self: expr => $action: expr) => {
        if $self.errors.len() == 0 {
            $action
        } else {
            return;
        }
    };
}

macro_rules! collect {
    ($self: expr => $t: expr) => {
        match $t {
            Ok(r) => r,

            Err(e) => {
                $self.errors.push(e);
                return;
            }
        }
    };
}

pub struct PrettyPrinter<W: Write> {
    writer: W,
    indent: u64,
    errors: Vec<io::Error>,
}

impl<W: Write> PrettyPrinter<W> {
    pub fn print(writer: W, module: &Module) -> io::Result<()> {
        let mut pp = PrettyPrinter {
            writer,
            indent: 0,
            errors: Vec::new(),
        };

        pp.visit_module(module, &());

        if pp.errors.len() > 0 {
            Err(pp.errors.remove(0))
        } else {
            Ok(())
        }
    }

    fn indent_left(&mut self) {
        if self.indent > 0 {
            self.indent -= 1;
        }
    }

    fn indent_write<'a, T: AsRef<[u8]>>(&mut self, data: T) {
        self.emit_indent();
        self.write(data);
    }

    fn emit_indent(&mut self) {
        for _ in 0..self.indent {
            no_err!(self => collect!(self => self.writer.write("\t".as_bytes())));
        }
    }

    fn write<'a, T: AsRef<[u8]>>(&mut self, data: T) {
        no_err!(self => collect!(self => self.writer.write(data.as_ref())));
    }

    fn indent_right(&mut self) {
        if self.indent < u64::MAX {
            self.indent += 1;
        }
    }
}

impl<W: Write> Visit for PrettyPrinter<W> {
    fn visit_ident(&mut self, i: &Ident, _: &dyn Node) {
        self.write(&*i.sym);
    }

    fn visit_object_lit(&mut self, n: &ObjectLit, _: &dyn Node) {
        match n.props.len() {
            0 => self.write("{}"),

            1..=3 => {
                self.write("{ ");
                for p in n.props.iter() {
                    indent!(self => {
                        self.visit_prop_or_spread(p, &());
                        self.write(", ");
                    });
                }
                self.write(" }");
            }

            _ => {
                self.write("{\n");
                for p in n.props.iter() {
                    indent!(self => {
                        self.visit_prop_or_spread(p, &());
                        self.write(",\n");
                    });
                }
                self.write("}");
            }
        }
    }

    fn visit_lit(&mut self, l: &Lit, _: &dyn Node) {
        match l {
            Lit::Str(ref s) => {
                self.write("\"");
                self.write(&*s.value);
                self.write("\"");
            }
            Lit::Bool(ref b) => self.write(&b.value.to_string()),
            Lit::Null(..) => self.write("null"),
            Lit::Num(ref number) => self.write(number.to_string()),
            Lit::BigInt(..) => todo!("BIGINT literal"),
            Lit::Regex(..) => todo!("REGEX literal"),
            Lit::JSXText(..) => todo!("JSXText literal"),
        }
    }

    fn visit_bin_expr(&mut self, n: &BinExpr, _: &dyn Node) {
        self.visit_expr(&*n.left, &());

        match n.op {
            BinaryOp::EqEqEq => self.write("==="),

            _ => todo!("binop: {:#?}", n.op),
        }

        self.visit_expr(&*n.right, &());
    }

    fn visit_assign_expr(&mut self, n: &AssignExpr, _: &dyn Node) {
        self.visit_pat_or_expr(&n.left, &());

        match n.op {
            AssignOp::Assign => self.write("="),
            _ => todo!("Emit {:#?}", n.op),
        }

        self.visit_expr(&n.right, &());
    }

    fn visit_call_expr(&mut self, n: &CallExpr, _: &dyn Node) {
        self.visit_expr_or_super(&n.callee, &());

        self.write("(");

        let last = n.args.len();
        for (index, arg) in n.args.iter().enumerate() {
            self.visit_expr_or_spread(arg, &());

            if index + 1 != last {
                self.write(",");
            }
        }

        self.write(")");
    }

    fn visit_pat(&mut self, p: &Pat, _: &dyn Node) {
        match p {
            Pat::Ident(ref i) => self.visit_ident(i, &()),

            Pat::Expr(ref e) => self.visit_expr(e, &()),

            Pat::Array(ArrayPat { ref elems, .. }) => {
                self.write("[");
                match elems.len() {
                    1 => {
                        if let Some(ref p) = elems.get(0).unwrap() {
                            self.visit_pat(p, &());
                        }
                    }

                    _ => todo!("Array pattern with length: {} ({:#?})", elems.len(), elems),
                }
                self.write("]");
            }
            ref p => todo!("Pattern: {:#?}", p),
        }
    }

    fn visit_member_expr(&mut self, n: &MemberExpr, _: &dyn Node) {
        self.visit_expr_or_super(&n.obj, &());

        if n.computed {
            self.write("[");
            self.visit_expr(&*n.prop, &());
            self.write("]");
        } else {
            self.write(".");
            self.visit_expr(&*n.prop, &());
        }
    }

    fn visit_fn_expr(&mut self, n: &FnExpr, _: &dyn Node) {
        self.write("function ");

        if let Some(ref ident) = n.ident {
            self.write(&*ident.sym);
        }

        self.write("(");
        for param in n.function.params.iter() {
            self.visit_pat(&param.pat, &());
            self.write(",");
        }
        self.write(") {\n");

        if let Some(ref body) = n.function.body {
            indent!(self => {
                for stmt in body.stmts.iter() {
                    self.visit_stmt(stmt, &());
                }
            });
        }

        self.indent_write("}");
    }

    fn visit_if_stmt(&mut self, n: &IfStmt, _: &dyn Node) {
        self.indent_write("if (");
        self.visit_expr(&n.test, &());
        self.write(")");

        match *n.cons {
            Stmt::Block(..) => self.visit_stmt(&n.cons, &()),
            _ => {
                self.write("{\n");
                indent!(self => {
                    self.visit_stmt(&n.cons, &());
                });
                self.indent_write("}")
            }
        }

        if let Some(ref fbranch) = n.alt {
            self.write(" else ");
            match **fbranch {
                Stmt::Block(..) => self.visit_stmt(fbranch, &()),
                _ => {
                    self.write("{\n");
                    indent!(self => {
                        self.visit_stmt(fbranch, &());
                    });
                    self.indent_write("}\n")
                }
            }
        } else {
            self.write("\n");
        }
    }

    fn visit_block_stmt(&mut self, n: &BlockStmt, _: &dyn Node) {
        self.write("{\n");

        indent!(self => {
            for stmt in n.stmts.iter() {
                self.visit_stmt(stmt, &());
            }
        });

        self.write("}");
    }

    fn visit_return_stmt(&mut self, n: &ReturnStmt, _: &dyn Node) {
        match n.arg {
            Some(ref e) => {
                self.indent_write("return ");
                self.visit_expr(e, &());
                self.write(";\n");
            }

            None => self.indent_write("return;\n"),
        }
    }

    fn visit_expr_stmt(&mut self, e: &ExprStmt, _: &dyn Node) {
        self.emit_indent();
        self.visit_expr(&*e.expr, &());
        self.write(";\n");
    }

    fn visit_var_decl(&mut self, n: &VarDecl, _: &dyn Node) {
        let modifier = match n.kind {
            VarDeclKind::Var => "var ",
            VarDeclKind::Let => "let ",
            VarDeclKind::Const => "const ",
        };

        self.indent_write(modifier);

        let last = n.decls.len();
        for (index, decl) in n.decls.iter().enumerate() {
            self.visit_pat(&decl.name, &());

            self.write(" = ");

            if let Some(ref init) = decl.init {
                self.visit_expr(&*init, &());
            }

            if index + 1 == last {
                self.write(";\n");
            } else {
                self.write(",\n");
            }
        }
    }
}
