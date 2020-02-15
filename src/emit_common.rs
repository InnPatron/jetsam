pub fn constuctor_name(index: usize, class_name: &str) -> String {
    format!("__new_{}_{}", class_name, index)
}
