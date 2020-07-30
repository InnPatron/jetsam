macro_rules! const_str {
    ($name: ident => $val: expr) => {
        pub const $name: &'static str = $val;
    }
}

macro_rules! gen_help_str {
    ($name: ident => $val: expr) => {
        pub const $name: &'static str = concat!("[GEN] ", $val);
    }
}

macro_rules! bool_values {
    () => {
        &["true", "false"]
    }
}

macro_rules! opt_arg {
    ($app: expr =>
     key: $key: expr;
     long: $long: expr;
     help: $help: expr) => {
        $app = $app.arg(Arg::with_name($key)
            .long($long)
            .help($help)
            .required(false));
    };

    ($app: expr =>
     key: $key: expr;
     long: $long: expr;
     help: $help: expr;
     help-long: $help_long: expr) => {
        $app = $app.arg(Arg::with_name($key)
            .long($long)
            .help($help)
            .long_help($help_long)
            .required(false));
    };


    ($app: expr =>
     key: $key: expr;
     long: $long: expr;
     values: $values: expr;
     validator: $validator: expr;
     help: $help: expr) => {
        $app = $app.arg(Arg::with_name($key)
            .long($long)
            .takes_value(true)
            .possible_values($values)
            .validator($validator)
            .help($help)
            .required(false));
    };

    ($app: expr =>
     key: $key: expr;
     long: $long: expr;
     values: $values: expr;
     validator: $validator: expr;
     help: $help: expr;
     help-long: $help_long: expr) => {
        $app = $app.arg(Arg::with_name($key)
            .long($long)
            .takes_value(true)
            .validator($validator)
            .possible_values($values)
            .help($help)
            .long_help($help_long)
            .required(false));
    }
}

macro_rules! extract_opt_arg {
    ($matches: expr =>
     key: $key: expr;
     converter: $converter: expr;
     =>
     gen_config: $gen_config: expr;
     gen_key: $gen_key: ident) => {
        (|| {
            if let Some(v) = $matches.value_of($key) {

                let v = match $converter(v) {
                    Ok(v) => v,

                    Err(e) => return Err(e),        // Hack to get type inference to work
                };
                $gen_config.$gen_key = v;
            }
            Ok(())
        })()
    }
}
