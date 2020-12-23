pub enum TsFlavor {
    TsNum,
    TsFull,
    TsCustom(TsFeatures),
}

impl TsFlavor {
    pub fn features(&self) -> TsFeatures {
        match *self {
            TsFlavor::TsNum => TsFeatures::ts_num(),
            TsFlavor::TsFull => TsFeatures::all(),
            TsFlavor::TsCustom(ref custom) => custom.clone(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Builder)]
pub struct TsFeatures {
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
    pub void_type: bool,

    #[builder(default = "false")]
    pub fn_type: bool,

    #[builder(default = "false")]
    pub null_type: bool,

    #[builder(default = "false")]
    pub undefined_type: bool,

    #[builder(default = "false")]
    pub never_type: bool,

    #[builder(default = "false")]
    pub interfaces: bool,

    #[builder(default = "false")]
    pub interface_extension: bool,

    #[builder(default = "false")]
    pub recursive_type: bool,

    #[builder(default = "false")]
    /// Ex: function foo (x: { a: number}) { .. }
    /// => type annotation for x is a type literal
    pub type_literal: bool,

    #[builder(default = "false")]
    /// Ex: interface Foo {
    ///     brand: "FooBrand"
    /// }
    pub literal_type: bool,

    #[builder(default = "false")]
    pub class_type: bool,

    #[builder(default = "false")]
    pub type_alias: bool,
}

impl TsFeatures {
    // NOTE: Need to manually keep this in sync
    pub fn all() -> Self {
        TsFeaturesBuilder::default()
            .number_type(true)
            .boolean_type(true)
            .string_type(true)
            .array_type(true)
            .tuple_type(true)
            .explicit_enum_type(true)
            .object_type(true)
            .any_type(true)
            .void_type(true)
            .fn_type(true)
            .null_type(true)
            .undefined_type(true)
            .never_type(true)
            .interfaces(true)
            .interface_extension(true)
            .recursive_type(true)
            .type_literal(true)
            .literal_type(true)
            .class_type(true)
            .type_alias(true)
            .build()
            .expect("empty failed")
    }

    pub fn empty() -> Self {
        TsFeaturesBuilder::default().build().expect("empty failed")
    }

    pub fn ts_num() -> Self {
        TsFeaturesBuilder::default()
            .number_type(true)
            .fn_type(true)
            .allow_simple_records()
            .fn_type(true)
            .build()
            .expect("ts_num failed")
    }
}

impl TsFeaturesBuilder {
    pub fn empty() -> Self {
        TsFeaturesBuilder::default()
    }

    pub fn allow_simple_records(&mut self) -> &mut Self {
        self.interfaces(true)
            .type_literal(true)
            .interface_extension(false)
            .recursive_type(false)
    }
}
