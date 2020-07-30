#[derive(Debug, Clone)]
pub struct GenConfig {

    /// Define functions that wrap class constructors (expiremental)
    /// Used by:
    ///     * TS-FULL
    pub output_constructor_wrappers: bool,

    /// Define 1:1 opaque nominal datatypes in Pyret per exported TS interface
    /// Used by:
    ///     * TS-FULL
    pub output_opaque_interfaces: bool,

    /// Define getter wraps for exported top-level variables. Defaults to true.
    /// Used by:
    ///     * TS-FULL
    ///     * TS-NUM
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
