use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GenConfig {

    /// Define functions that wrap class constructors (expiremental)
    /// Used by:
    ///     * TS-FULL
    /// Serde name needs to be kept in sync with `OPTION_CONSTRUCTOR_WRAPPERS`
    #[serde(rename = "constructor-wrappers")]
    pub output_constructor_wrappers: bool,

    /// Define 1:1 opaque nominal datatypes in Pyret per exported TS interface
    /// Used by:
    ///     * TS-FULL
    /// Serde name needs to be kept in sync with `OPTION_OPAQUE_INTERFACES`
    #[serde(rename = "opaque-interfaces")]
    pub output_opaque_interfaces: bool,

    /// Define getter wraps for exported top-level variables. Defaults to true.
    /// Used by:
    ///     * TS-FULL
    ///     * TS-NUM
    /// Serde name needs to be kept in sync with `OPTION_WRAP_TOP_LEVEL_VARS`
    #[serde(rename = "wrap-top-level-vars")]
    pub wrap_top_level_vars: bool,

}

impl Default for GenConfig {
    fn default() -> Self {
        GenConfig {
            output_constructor_wrappers: true,
            output_opaque_interfaces: true,
            wrap_top_level_vars: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EmitConfig {
    pub json: bool,
    pub js: bool,
}
