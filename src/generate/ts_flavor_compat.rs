use crate::ts::TsFeatures;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CompatError {
    RecursiveType,
    BoolType,
    NumberType,
    StringType,
    VoidType,
    ArrayType,
    TupleType,
    NeverType,
    AnyType,
    InterfaceType,
    TypeLiteral,
    LiteralType,
    UndefinedType,
}

macro_rules! basic_check {
    ($detected: expr, $target: expr, $field: ident @ LTE [$e: expr => $errors: expr]) => {{
        if $detected.$field && $target.$field == false {
            $errors.push($e);
        }
    }};
}

pub fn compatible(detected: &TsFeatures, target: &TsFeatures) -> Result<(), Vec<CompatError>> {
    let mut errors = Vec::new();

    basic_check!(detected, target, number_type      @ LTE [CompatError::NumberType => errors]);
    basic_check!(detected, target, boolean_type     @ LTE [CompatError::BoolType => errors]);
    basic_check!(detected, target, string_type      @ LTE [CompatError::StringType => errors]);
    basic_check!(detected, target, void_type        @ LTE [CompatError::VoidType => errors]);
    basic_check!(detected, target, array_type       @ LTE [CompatError::ArrayType => errors]);
    basic_check!(detected, target, tuple_type       @ LTE [CompatError::TupleType => errors]);
    basic_check!(detected, target, never_type       @ LTE [CompatError::NeverType => errors]);
    basic_check!(detected, target, undefined_type   @ LTE [CompatError::UndefinedType => errors]);
    basic_check!(detected, target, any_type         @ LTE [CompatError::AnyType => errors]);
    basic_check!(detected, target, type_literal     @ LTE [CompatError::TypeLiteral => errors]);
    basic_check!(detected, target, literal_type     @ LTE [CompatError::LiteralType => errors]);

    if errors.len() == 0 {
        Ok(())
    } else {
        Err(errors)
    }
}
