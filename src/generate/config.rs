pub struct GenConfig {

    /// Define functions that wrap class constructors (expiremental)
    /// Used by:
    ///     * TS-FULL
    pub output_constructor_wrappers: bool,

    /// Define 1:1 opaque nominal datatypes in Pyret per exported TS interface
    /// Used by:
    ///     * TS-FULL
    pub output_opaque_interfaces: bool,

}

impl Default for GenConfig {
    fn default() -> Self {
        GenConfig {
            output_constructor_wrappers: true,
            output_opaque_interfaces: true,
        }
    }
}

pub struct EmitConfig {
    pub json: bool,
    pub js: bool,
}
