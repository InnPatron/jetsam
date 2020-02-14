macro_rules! get_dep_src {
    ($self: expr, $src_str: expr) => {
        $self.dependency_map.get(&*$src_str.value).expect("Source path not found in dependency_map")
    }
}

macro_rules! opt {
    ($options: expr, $opt:ident, $body: block) => {
        if $options.$opt {
            $body
        }
    }
}
