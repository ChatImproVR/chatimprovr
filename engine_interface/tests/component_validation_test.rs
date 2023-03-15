use cimvr_engine_interface::{self, component_validation::is_fixed_size};
use serde::{Deserialize, Serialize};

#[test]
#[should_panic]
fn ser_string() {
    let a = "Hello, world!".to_string();
    cimvr_engine_interface::is_fixed_size(a).unwrap();
}

#[test]
fn ser_bool() {
    let a = true;
    is_fixed_size(a).unwrap();
}

#[test]
fn ser_i8() {
    let a = 0i8;
    is_fixed_size(a).unwrap();
}

#[test]
fn ser_i16() {
    let a = 0i16;
    is_fixed_size(a).unwrap();
}

#[test]
fn ser_i32() {
    let a = 0i32;
    is_fixed_size(a).unwrap();
}

#[test]
fn ser_i64() {
    let a = 0i64;
    is_fixed_size(a).unwrap();
}

#[test]
fn ser_u8() {
    let a = 0u8;
    is_fixed_size(a).unwrap();
}

#[test]
fn ser_u16() {
    let a = 0u16;
    is_fixed_size(a).unwrap();
}

#[test]
fn ser_u32() {
    let a = 0u32;
    is_fixed_size(a).unwrap();
}

#[test]
fn ser_u64() {
    let a = 0u64;
    is_fixed_size(a).unwrap();
}

#[test]
fn ser_f32() {
    let a = 0f32;
    is_fixed_size(a).unwrap();
}

#[test]
fn ser_f64() {
    let a = 0f64;
    is_fixed_size(a).unwrap();
}

#[test]
fn ser_char() {
    let a = 'a';
    is_fixed_size(a).unwrap();
}

#[test]
#[should_panic]
fn ser_str() {
    let a = "Hello, world!";
    is_fixed_size(a).unwrap();
}

#[test]
fn ser_bytes_known_size() {
    // Shouldn't panic, because it's a known sized byte array.
    let a = b"Fuck you!";
    is_fixed_size(a).unwrap();
}

#[test]
#[should_panic]
fn ser_bytes_variable_size() {
    let a = b"Fuck you!".as_slice();
    is_fixed_size(a).unwrap();
}

#[test]
#[should_panic]
fn ser_none() {
    let a: Option<()> = None;
    is_fixed_size(a).unwrap();
}

#[test]
#[should_panic]
fn ser_some() {
    let a: Option<()> = Some(());
    is_fixed_size(a).unwrap();
}

#[test]
fn ser_unit() {
    let a = ();
    is_fixed_size(a).unwrap();
}

#[test]
fn ser_unit_struct() {
    #[derive(Serialize, Deserialize, Debug, Clone, Copy)]
    struct UnitStruct;

    let a = UnitStruct;
    is_fixed_size(a).unwrap();
}

#[test]
fn ser_unit_variant() {
    #[derive(Serialize, Deserialize, Debug, Clone, Copy)]
    enum UnitVariant {
        Unit,
    }

    let a = UnitVariant::Unit;
    is_fixed_size(a).unwrap();
}

#[test]
fn ser_newtype_struct() {
    #[derive(Serialize, Deserialize, Debug, Clone, Copy)]
    struct NewtypeStruct(u8);

    let a = NewtypeStruct(0);
    is_fixed_size(a).unwrap();
}

#[test]
#[should_panic]
fn ser_newtype_variant() {
    #[derive(Serialize, Deserialize, Debug, Clone, Copy)]
    enum NewtypeVariant {
        Newtype(u8),
        Unit,
    }

    let a = NewtypeVariant::Newtype(0);
    is_fixed_size(a).unwrap();
}

#[test]
#[should_panic]
fn ser_seq() {
    let a = vec![0; 10];
    is_fixed_size(a).unwrap();
}

#[test]
fn ser_tuple_fixed() {
    let a = (0, 0);
    is_fixed_size(a).unwrap();
}

#[test]
#[should_panic]
fn ser_tuple_variable() {
    // If this works, we've ROYALLY fucked up. :)
    let a = (
        vec![0; 10],
        "Too much time has been spent",
        b"doing this shit",
        "I'm losing my mind".to_string(),
    );
    is_fixed_size(a).unwrap();
}

#[test]
fn ser_tuple_struct_fixed() {
    #[derive(Serialize, Deserialize, Debug, Clone, Copy)]
    struct TupleStruct(u8, u8);

    let a = TupleStruct(0, 0);
    is_fixed_size(a).unwrap();
}

#[test]
#[should_panic]
fn ser_tuple_struct_variable() {
    #[derive(Serialize, Deserialize, Debug, Clone)]
    struct TupleStruct(Vec<u8>, String);

    let a = TupleStruct(vec![0; 10], "I wanna kms".to_string());
    is_fixed_size(a).unwrap();
}

#[test]
#[should_panic]
fn ser_tuple_variant() {
    #[derive(Serialize, Deserialize, Debug, Clone)]
    enum TupleVariant {
        Tuple(u8, u8),
        Unit,
    }
    // Should panic because we can't serialize a tuple variant.
    // Cuz fuck you, that's why.
    let a = TupleVariant::Tuple(0, 0);
    is_fixed_size(a).unwrap();
}

#[test]
#[should_panic]
fn ser_map() {
    use std::collections::HashMap;
    let mut a = HashMap::new();
    a.insert(0, 0);
    is_fixed_size(a).unwrap();
}

#[test]
fn ser_struct_fixed() {
    #[derive(Serialize, Deserialize, Debug, Clone)]
    struct Struct {
        a: u8,
    }

    let a = Struct { a: 0 };
    is_fixed_size(a).unwrap();
}

#[test]
#[should_panic]
fn ser_struct_variable() {
    #[derive(Serialize, Deserialize, Debug, Clone)]
    struct VariableStruct {
        a: String,
    }

    let a = VariableStruct {
        a: "I regret nothing. Removing size parameter was based.".to_string(),
    };
    is_fixed_size(a).unwrap();
}

#[test]
#[should_panic]
fn ser_struct_variant() {
    #[derive(Serialize, Deserialize, Debug, Clone)]
    enum StructVariant {
        Struct { a: u8 },
        Unit,
    }

    let a = StructVariant::Struct { a: 0 };
    is_fixed_size(a).unwrap();
}
