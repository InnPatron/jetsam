use super::ts::TsFlavor;

pub const TS_NUM_STRINGS: &'static [&'static str] = &[
    "ts-num",
    "TS-NUM",
];

pub const TS_FULL_STRINGS: &'static [&'static str] = &[
    "ts-full",
    "TS-FULL",
];

pub const TS_FLAVOR_STRINGS: &'static [&'static str] = &[
    "ts-num",
    "TS-NUM",

    "ts-full",
    "TS-FULL",
];

pub const DEFAULT_TS_FLAVOR: (TsFlavor, &'static str) = (TsFlavor::TsNum, "TS-NUM");

const_str!(OPTION_TS_FLAVOR => "ts-flavor");

// Needs to be kept in sync with `GenConfig.output_constructor_wrappers` serde name
const_str!(OPTION_CONSTRUCTOR_WRAPPERS => "constructor-wrappers");

// Needs to be kept in sync with `GenConfig.output_opaque_interfaces` serde name
const_str!(OPTION_OPAQUE_INTERFACES => "opaque-interfaces");

// Needs to be keep in sync with `GenConfig.wrap_top_level_vars` serde name
const_str!(OPTION_WRAP_TOP_LEVEL_VARS => "wrap-top-level-vars");
