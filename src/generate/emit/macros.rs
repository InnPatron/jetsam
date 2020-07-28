macro_rules! ident {
    ($bind: expr) => {
        Ident {
            span: DUMMY_SP,
            sym: JsWord::from($bind),
            type_ann: None,
            optional: false,            // Indicates optional argument (ignore)
        }
    }
}

macro_rules! param {
    ($id: expr) => {
        Param {
            span: DUMMY_SP,
            decorators: vec![],
            pat: Pat::Ident($id),
        }
    }
}

macro_rules! stmt {
    (return) => {
        Stmt::Return(ReturnStmt { span: DUMMY_SP, arg: None })
    };

    (return $v: expr) => {
        Stmt::Return(ReturnStmt { span: DUMMY_SP, arg: Some(Box::new($v)) })
    };

    (var $bind: expr => $value: expr) => {{
        stmt!([VarDeclKind::VAR], $bind => $value)
    }};

    (let $bind: expr => $value: expr) => {{
        stmt!([VarDeclKind::Let], $bind => $value)
    }};

    (const $bind: expr => $value: expr) => {{
        stmt!([VarDeclKind::Const], $bind => $value)
    }};

    (block $($stmt: expr);*) => {
        Stmt::Block(BlockStmt {
            span: DUMMY_SP,
            stmts: vec![$($stmt),*],
        })
    };

    (if $test: expr => $branch: expr) => {
        Stmt::If(IfStmt {
            span: DUMMY_SP,
            test: Box::new($test),
            cons: Box::new($branch),
            alt: None
        })
    };

    (if $test: expr => $tbranch: expr; else => $fbranch: expr) => {
        Stmt::If(IfStmt {
            span: DUMMY_SP,
            test: Box::new($test),
            cons: Box::new($tbranch),
            alt: Some(Box::new($fbranch)),
        })
    };

    ([$kind: expr], $bind: expr => $value: expr) => {
        Stmt::Decl(Decl::Var(VarDecl {
            span: DUMMY_SP,
            kind: $kind,
            declare: true,                  // TODO: What is this for?
            decls: vec![
                VarDeclarator {
                    span: DUMMY_SP,
                    name: Pat::Ident($bind),
                    init: Some(Box::new($value)),
                    definite: true,         // TODO: What is this for?
                }
            ]
        }))
    }
}

macro_rules! expr {
    (Ident $ident: expr) => {
        Expr::Ident(ident!($ident))
    };

    (Fn $fn: expr) => {
        Expr::Fn(FnExpr {
            ident: None,
            function: $fn
        })
    };

    (Fn($ident: expr) @ $fn: expr) => {
        Expr::Fn(FnExpr {
            ident: Some(ident!($ident)),
            function: $fn
        })
    };

    (Call $fn: expr) => {
        Expr::Call(CallExpr {
            span: DUMMY_SP,
            callee: ExprOrSuper::Expr(Box::new($fn)),
            args: vec![],
            type_args: None,
        })
    };

    (Call $fn: expr => $($arg: expr),+) => {
        Expr::Call(CallExpr {
            span: DUMMY_SP,
            callee: ExprOrSuper::Expr(Box::new($fn)),
            args: vec![$(ExprOrSpread {
                spread: None,
                expr: Box::new($arg)
            }),+],
            type_args: None,
        })
    };

    (=== $lhs: expr, $rhs: expr) => {
        Expr::Bin(BinExpr {
            span: DUMMY_SP,
            op: BinaryOp::EqEqEq,
            left: Box::new($lhs),
            right: Box::new($rhs),
        })
    };

    (DOT $object: expr => $member: expr) => {
        Expr::Member(MemberExpr {
            span: DUMMY_SP,
            obj: ExprOrSuper::Expr(Box::new($object)),
            prop: Box::new($member),
            computed: false,
        })
    }
}


macro_rules! function {
    ($($arg: expr),* => $($stmt: expr);+) => {
        Function {
            params: vec![$($arg),*],
            decorators: vec![],
            span: DUMMY_SP,
            body: Some(BlockStmt {
                span: DUMMY_SP,
                stmts: vec![$($stmt),*],
            }),
            is_generator: false,
            is_async: false,
            type_params: None,
            return_type: None,
        }

    }
}
