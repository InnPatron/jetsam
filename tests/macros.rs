macro_rules! include_test {
    ($file: ident) => {
        include_str!(concat!("./data/", $file))
    }
}
