#[derive(Debug, PartialEq, Eq, Clone, Builder)]
pub struct TsFlavor {
    #[builder(default = "false")]
    pub number_type: bool,

    #[builder(default = "false")]
    pub boolean_type: bool,

    #[builder(default = "false")]
    pub string_type: bool,

    #[builder(default = "false")]
    pub array_type: bool,

    #[builder(default = "false")]
    pub tuple_type: bool,

    #[builder(default = "false")]
    pub explicit_enum_type: bool,

    #[builder(default = "false")]
    pub object_type: bool,

    #[builder(default = "false")]
    pub any_type: bool,

    #[builder(default = "false")]
    pub fn_type: bool,

    #[builder(default = "false")]
    pub fn_declarations: bool,

    #[builder(default = "false")]
    pub interfaces: bool,

    #[builder(default = "false")]
    pub interface_extension: bool,

    #[builder(default = "false")]
    pub recursive_interface: bool,

    #[builder(default = "false")]
    /// Ex: funciton foo (x: { a: number}) { .. }
    /// => type annotation for x is a type literal
    pub type_literal: bool,

    #[builder(default = "false")]
    pub literal_type: bool,
}

impl TsFlavor {

    pub fn empty() -> Self {
        TsFlavorBuilder::default()
            .build()
            .expect("empty failed")
    }

    pub fn ts_num() -> Self {
        TsFlavorBuilder::default()
            .number_type(true)
            .fn_type(true)
            .allow_simple_records()
            .fn_declarations(true)
            .build()
            .expect("ts_num failed")
    }
}

impl TsFlavorBuilder {
    pub fn allow_simple_records(&mut self) -> &mut Self {
        self
            .interfaces(true)
            .type_literal(true)
            .interface_extension(false)
            .recursive_interface(false)
    }
}
