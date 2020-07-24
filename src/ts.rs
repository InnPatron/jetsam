#[derive(Debug, PartialEq, Eq, Clone, Builder)]
pub struct TsFlavor {
    #[builder(default = "false")]
    pub number: bool,
}

impl TsFlavor {

    pub fn ts_num() -> Self {
        TsFlavorBuilder::default()
            .number(true)
            .build()
            .expect("ts_num failed")
    }
}
