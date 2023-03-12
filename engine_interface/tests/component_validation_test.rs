use cimvr_engine_interface::{
    self, component_validation::CustomSerializer,
};
use serde::{Deserialize, Serialize};

#[test]
#[should_panic]
fn ser_string() {
    let a = "Hello, world!".to_string();
    a.serialize(CustomSerializer::new()).unwrap();
}

#[test]
fn ser_bool() {
    let a = true;
    a.serialize(CustomSerializer::new()).unwrap();
}

#[test]
fn ser_i8() {
    let a = 0i8;
    a.serialize(CustomSerializer::new()).unwrap();
}

#[test]
fn ser_i16() {
    let a = 0i16;
    a.serialize(CustomSerializer::new()).unwrap();
}

#[test]
fn ser_i32() {
    let a = 0i32;
    a.serialize(CustomSerializer::new()).unwrap();
}

#[test]
fn ser_i64() {
    let a = 0i64;
    a.serialize(CustomSerializer::new()).unwrap();
}

#[test]
fn ser_u8() {
    let a = 0u8;
    a.serialize(CustomSerializer::new()).unwrap();
}

#[test]
fn ser_u16() {
    let a = 0u16;
    a.serialize(CustomSerializer::new()).unwrap();
}

#[test]
fn ser_u32() {
    let a = 0u32;
    a.serialize(CustomSerializer::new()).unwrap();
}

#[test]
fn ser_u64() {
    let a = 0u64;
    a.serialize(CustomSerializer::new()).unwrap();
}

#[test]
fn ser_f32() {
    let a = 0f32;
    a.serialize(CustomSerializer::new()).unwrap();
}

#[test]
fn ser_f64() {
    let a = 0f64;
    a.serialize(CustomSerializer::new()).unwrap();
}

#[test]
fn ser_char() {
    let a = 'a';
    a.serialize(CustomSerializer::new()).unwrap();
}

#[test]
#[should_panic]
fn ser_str() {
    let a = "Hello, world!";
    a.serialize(CustomSerializer::new()).unwrap();
}

#[test]
#[should_panic]
fn ser_bytes() {
    let a = b"Fuck you!";
    a.serialize(CustomSerializer::new()).unwrap();
}

#[test]
#[should_panic]
fn ser_none() {
    let a: Option<()> = None;
    a.serialize(CustomSerializer::new()).unwrap();
}

#[test]
#[should_panic]
fn ser_some() {
    let a: Option<()> = Some(());
    a.serialize(CustomSerializer::new()).unwrap();
}

#[test]
fn ser_unit() {
    let a = ();
    a.serialize(CustomSerializer::new()).unwrap();
}

#[test]
fn ser_unit_struct() {
    #[derive(Serialize, Deserialize, Debug, Clone, Copy)]
    struct UnitStruct;

    let a = UnitStruct;
    a.serialize(CustomSerializer::new()).unwrap();
}



#[test]
fn ser_unit_variant() {
    #[derive(Serialize, Deserialize, Debug, Clone, Copy)]
    enum UnitVariant {
        Unit,
    }

    let a = UnitVariant::Unit;
    a.serialize(CustomSerializer::new()).unwrap();
}

#[test]
fn ser_newtype_struct() {
    #[derive(Serialize, Deserialize, Debug, Clone, Copy)]
    struct NewtypeStruct(u8);

    let a = NewtypeStruct(0);
    a.serialize(CustomSerializer::new()).unwrap();
}

#[test]
#[should_panic]
fn ser_newtype_variant() {
    #[derive(Serialize, Deserialize, Debug, Clone, Copy)]
    enum NewtypeVariant {
        Newtype(u8),
        Unit
    }

    let a = NewtypeVariant::Newtype(0);
    a.serialize(CustomSerializer::new()).unwrap();
}

#[test]
#[should_panic]
fn ser_seq () {
    let a = vec![0; 10];
    a.serialize(CustomSerializer::new()).unwrap();
}

#[test]
fn ser_tuple_fixed() {
    let a = (0, 0);
    a.serialize(CustomSerializer::new()).unwrap();
}

#[test]
#[should_panic]
fn ser_tuple_variable() {
    // If this works, we've ROYALLY fucked up. :)
    let a = (vec![0; 10], "Too much time has been spent", b"doing this shit", "I'm losing my mind".to_string());
    a.serialize(CustomSerializer::new()).unwrap();
}

#[test]
fn ser_tuple_struct_fixed() {
    #[derive(Serialize, Deserialize, Debug, Clone, Copy)]
    struct TupleStruct(u8, u8);

    let a = TupleStruct(0, 0);
    a.serialize(CustomSerializer::new()).unwrap();
}

#[test]
#[should_panic]
fn ser_tuple_struct_variable() {
    #[derive(Serialize, Deserialize, Debug, Clone)]
    struct TupleStruct(Vec<u8>, String);

    let a = TupleStruct(vec![0; 10], "I wanna kms".to_string());
    a.serialize(CustomSerializer::new()).unwrap();
}

#[test]
#[should_panic]
fn ser_tuple_variant() {
    #[derive(Serialize, Deserialize, Debug, Clone)]
    enum TupleVariant {
        Tuple(u8, u8),
        Unit
    }
    // Should panic because we can't serialize a tuple variant.
    // Cuz fuck you, that's why.
    let a = TupleVariant::Tuple(0, 0);
    a.serialize(CustomSerializer::new()).unwrap();
}

#[test]
#[should_panic]
fn ser_map() {
    use std::collections::HashMap;
    let mut a = HashMap::new();
    a.insert(0, 0);
    a.serialize(CustomSerializer::new()).unwrap();
}

#[test]
fn ser_struct_fixed() {
    #[derive(Serialize, Deserialize, Debug, Clone)]
    struct Struct {
        a: u8,
    }

    let a = Struct { a: 0 };
    a.serialize(CustomSerializer::new()).unwrap();
}

#[test]
#[should_panic]
fn ser_fixed_variable() {
    #[derive(Serialize, Deserialize, Debug, Clone)]
    struct VariableSize {
        a: String,
    }

    let a = VariableSize { a: "I regret nothing. Removing size parameter was based.".to_string() };
    a.serialize(CustomSerializer::new()).unwrap();
}

#[test]
#[should_panic]
fn ser_struct_variant() {
    #[derive(Serialize, Deserialize, Debug, Clone)]
    enum StructVariant {
        Struct { a: u8 },
        Unit
    }

    let a = StructVariant::Struct { a: 0 };
    a.serialize(CustomSerializer::new()).unwrap();
}
