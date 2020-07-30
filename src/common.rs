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

const_str!(OPTIONS_BASE_CONFIG => "base-config");

//
// ======================
// Codegen option strings
// ======================
//

const_str!(OPTION_TS_FLAVOR => "ts-flavor");

// Needs to be kept in sync with `GenConfig.output_constructor_wrappers` serde name
const_str!(OPTION_CONSTRUCTOR_WRAPPERS => "constructor-wrappers");

// Needs to be kept in sync with `GenConfig.output_opaque_interfaces` serde name
const_str!(OPTION_OPAQUE_INTERFACES => "opaque-interfaces");

// Needs to be keep in sync with `GenConfig.wrap_top_level_vars` serde name
const_str!(OPTION_WRAP_TOP_LEVEL_VARS => "wrap-top-level-vars");

//
// ============
// Help strings
// ============
//
const_str!(OPTION_TS_FLAVOR_HELP =>
    "TypeScript subset to accept as input"
);

const_str!(OPTION_REQUIRE_PATH_HELP =>
"Import path of the TS implementation file relative to the generated bindings file [default: Same directory as the generated bindings file]"
);

gen_help_str!(OPTION_CONSTRUCTOR_WRAPPERS_HELP =>
"Generate Pyret functions around class constructors"
);

gen_help_str!(OPTION_CONSTRUCTOR_WRAPPERS_HELP_LONG =>
"Generate Pyret functions around class constructors.
Used by:
    * TS-FULL

[default: true]
"
);

gen_help_str!(OPTION_OPAQUE_INTERFACES_HELP =>
"Generate 1:1 opaque nominal datatypes for Pyret interfaces"
);

gen_help_str!(OPTION_OPAQUE_INTERFACES_HELP_LONG =>
"Generate 1:1 opaque nominal datatypes for Pyret interfaces
Used by:
    * TS-FULL
[default: true]
"
);

gen_help_str!(OPTION_WRAP_TOP_LEVEL_VARS_HELP =>
"Generate converter getters around exported top-level variables"
);

gen_help_str!(OPTION_WRAP_TOP_LEVEL_VARS_HELP_LONG =>
"Generate converter getters around exported top-level variables
Used by:
    * TS-FULL
    * TS-NUM
[default: true]
"
);
