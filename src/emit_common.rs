pub fn constuctor_name(index: usize, class_name: &str) -> String {
    format!("__new-{}{}", class_name, index)
}
