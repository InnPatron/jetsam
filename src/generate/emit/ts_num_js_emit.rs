use std::path::Path;

use swc_atoms::JsWord;
use swc_common::DUMMY_SP;
use swc_ecma_ast::*;
use swc_ecma_ast::Module as AstModule;

use indexmap::IndexMap;

use crate::generate::type_structs::*;
use crate::generate::error::EmitError;
use crate::compile_opt::CompileOpt;

use super::JsEmitter;

const C_TS_NUMBER_PY_NUMBER: &'static str = "C_ts_number_py_number";
const C_PY_NUMBER_TS_NUMBER: &'static str = "C_py_number_ts_number";

macro_rules! root_value {
    ($i: expr) => {
        format!("root[\"{}\"]", $i)
    }
}

#[derive(Debug, Clone, Copy)]
enum Conversion {
    TsToPy,
    PyToTs,
}

pub(super) struct TsNumJsOutput<'a> {
    options: &'a CompileOpt<'a>,
    overrides: IndexMap<String, Expr>,
    anon_counter: u64,
}

impl<'a> TsNumJsOutput<'a> {
    pub fn new(options: &'a CompileOpt<'a>) -> Self {
        TsNumJsOutput {
            options,
            overrides: IndexMap::new(),
            anon_counter: 0,
        }
    }

    fn anon_inc(&mut self) -> u64 {
        self.anon_counter += 1;
        self.anon_counter
    }

    fn tmp_binding(&mut self) -> String {
        format!("___{}", self.anon_inc())
    }

    fn prelude(&self, body: &mut Vec<ModuleItem>, require_path: &str) {
        // const root = require(require_path);
        let root_import = {
            let call = expr!(Call expr!(Ident "require") => expr!(String require_path));
            stmt!(const "root" => call)
        };

        // module.exports = Object.assign({}, root)
        let default_set = {
            let object_dot = expr!(Member expr!(Ident "Object") => expr!(String "assign"));
            let object_assign_call = expr!(Call object_dot =>
                expr!(Object),
                expr!(Ident "root")
            );

            let module_dot = expr!(Member expr!(Ident "module") => expr!(String "exports"));
            let module_dot = pat!(expr module_dot);
            let assign = expr!(Assign module_dot => object_assign_call);

            stmt!(Expr assign)
        };



        let c_ts_number_py_number = function!(
            param!(ident!("ts_num"))
            =>
            stmt!(return expr!(Ident "ts_num"))
        );

        let c_ts_number_py_number = stmt!(
            const "C_ts_number_py_number" =>
                expr!(Fn("C_ts_number_py_number") @ c_ts_number_py_number)
        );

        let condition = expr!(Call expr!(Ident "typeof") =>
            expr!(Ident "py_num")
        );

        let c_py_number_ts_number = function!(
            param!(ident!("py_num"))
            =>
            stmt!(if expr!(=== condition, expr!(String "number"))
                => stmt!(return expr!(Ident "py_num"));
                else => stmt!(return expr!(Call
                        // Need to use `py_num.toFixnum` b/c need to search prototype chain
                        expr!(DOT expr!(Ident "py_num") => expr!(Ident "toFixnum"))))
            )
        );

        let c_py_number_ts_number = stmt!(
            const "C_py_number_ts_number" =>
                expr!(Fn("C_py_number_ts_number") @ c_py_number_ts_number)
        );

        body.push(ModuleItem::Stmt(root_import));
        body.push(ModuleItem::Stmt(default_set));
        body.push(ModuleItem::Stmt(c_ts_number_py_number));
        body.push(ModuleItem::Stmt(c_py_number_ts_number));
    }

    fn c_ts_number_py_number(&self, binding: &str) -> Expr {
        expr!(Call expr!(Ident C_TS_NUMBER_PY_NUMBER) => expr!(Ident binding))
    }

    fn c_py_number_ts_number(&self, binding: &str) -> Expr {
        expr!(Call expr!(Ident C_PY_NUMBER_TS_NUMBER) => expr!(Ident binding))
    }

    fn c_fn(&mut self,
        conversion: Conversion,
        fn_type: &FnType,
        binding: &str
    ) -> Expr {

        let result_id = format!("_result{}", self.anon_inc());
        let mut params: Vec<Param> = Vec::new();
        let mut body: Vec<Stmt> = Vec::new();
        let mut result_call_args: Vec<ExprOrSpread> = Vec::new();

        for (index, param_type) in fn_type.params.iter().enumerate() {
            let param_id = self.tmp_binding();
            let converted_id = self.tmp_binding();

            params.push(param!(ident!(param_id.as_str())));
            result_call_args.push(ExprOrSpread {
                spread: None,
                expr: Box::new(expr!(Ident converted_id.as_str()))
            });

            let converted = match (conversion, param_type) {
                (Conversion::TsToPy, Type::Fn(ref inner_fn_type)) => {
                    self.c_fn(Conversion::PyToTs, inner_fn_type, &param_id)
                }

                (Conversion::TsToPy, Type::Number) => self.c_py_number_ts_number(&param_id),

                (Conversion::PyToTs, Type::Fn(ref inner_fn_type)) => {
                    self.c_fn(Conversion::TsToPy, inner_fn_type, &param_id)
                }

                (Conversion::PyToTs, Type::Number) => self.c_ts_number_py_number(&param_id),

                ref t => unreachable!("Invalid type: {} {:?} (param {})", binding, t, index),
            };

            body.push(stmt!(let converted_id => converted));
        }

        let result = stmt!(let result_id.clone() =>
            expr!(Call-flat expr!(Ident binding) => result_call_args)
        );

        let return_conversion = match (conversion, &*fn_type.return_type) {
            (Conversion::TsToPy, &Type::Fn(ref fn_type)) => {
                self.c_fn(Conversion::TsToPy, fn_type, &result_id)
            }

            (Conversion::TsToPy, &Type::Number) => self.c_ts_number_py_number(&result_id),

            (Conversion::PyToTs, &Type::Fn(ref fn_type)) => {
                self.c_fn(Conversion::PyToTs, fn_type, &result_id)
            }

            (Conversion::PyToTs, &Type::Number) => self.c_py_number_ts_number(&result_id),


            ref t => unreachable!("c_ts_fn_py_fn: {} {:?} (return)", binding, t),
        };
        let return_stmt = stmt!(return return_conversion);
        body.push(result);
        body.push(return_stmt);

        let wrapper = Function {
            params,
            decorators: vec![],
            span: DUMMY_SP,
            body: Some(BlockStmt {
                span: DUMMY_SP,
                stmts: body,
            }),
            is_generator: false,
            is_async: false,
            type_params: None,
            return_type: None,
        };

        Expr::Fn(FnExpr {
            ident: None,
            function: wrapper
        })
    }
}

impl<'a> JsEmitter for TsNumJsOutput<'a> {
    fn handle_value(&mut self, current_module: &Path, name: &str, value_type: &Type)
        -> Result<(), EmitError> {

        match value_type {
            Type::Number => {
                let converted_value: Expr = self.c_ts_number_py_number(&root_value!(name));

                let overide = if self.options.gen_config.wrap_top_level_vars {
                    expr!(Fn function!(
                        =>
                        stmt!(return converted_value)
                    ))
                } else {
                    converted_value
                };

                self.overrides.insert(name.to_string(), overide);

                Ok(())
            }

            Type::Fn(ref fn_type) => {
                let value = self.c_fn(Conversion::TsToPy, fn_type, &root_value!(name));
                self.overrides.insert(name.to_string(), value);
                Ok(())
            }

            _ => Err(EmitError::Misc(
                    current_module.to_owned(),
                    format!("TS-NUM does not support values of type: {:?}", value_type)
                )),
        }
    }

    fn handle_type(&mut self, _current_module: &Path, _name: &str, _typ: &Type)
        -> Result<(), EmitError> {

        Ok(())
    }

    fn finalize(self, current_module: &Path)
        -> Result<AstModule, EmitError> {

        let mut body = Vec::new();
        self.prelude(&mut body, &self.options.require_path);

        for (override_key, override_value) in self.overrides.into_iter() {

            // module.exports["override_key"] = override_value;
            let override_stmt: Stmt = {
                let module_dot = expr!(Member expr!(Ident "module") => expr!(String "exports"));
                let module_override = expr!(Member module_dot =>
                    expr!(String override_key)
                );

                let assign = expr!(Assign-expr module_override => override_value);
                stmt!(Expr assign)
            };

            body.push(ModuleItem::Stmt(override_stmt));
        }

        Ok(AstModule {
            span: DUMMY_SP,
            body,
            shebang: None,
        })
    }
}
