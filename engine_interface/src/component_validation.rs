#![allow(unreachable_code)]
use crate::component_validate_error::ValidationError;
use serde;
use serde::ser::{
    SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
    SerializeTupleStruct, SerializeTupleVariant,
};
use serde::Serialize;

/// Returns `Ok(())` if the given data is represented by a fixed-size data structure.
/// The condition for fixed-sizedness is based on `bincode`'s representation.
#[track_caller]
pub fn is_fixed_size<V: Serialize>(v: V) -> Result<(), ValidationError> {
    v.serialize(FixedSizeValidator)
}

/// A serde "Serializer", which returns an error if the
/// data structure is not known to be fixed-size.
pub struct FixedSizeValidator;

impl FixedSizeValidator {
    pub fn new() -> Self {
        Self
    }
}

impl serde::Serializer for FixedSizeValidator {
    type Ok = ();
    type Error = ValidationError;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    // Fixed size
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(Self)
    }

    // Variable size
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(ValidationError)
    }

    // Variable size
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(ValidationError)
    }
    // Fixed size
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(Self)
    }
    // Fixed size, legit doesn't contain data.
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
    // Fixed size, idfk man. These things dont hold data.
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
    // This is a fixed size. It's just a wrapper around a single value. The value is what matters and can be variable size.
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Ok(())
    }
    // Fixed size. It's a wrapper around multiple values, which could potentially have a size.
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(Self)
    }
    // Not a fixed size. We're not getting into fuckery with enums that have variants that could POTENTIALLY have data. Sugma.
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(ValidationError)
    }

    // Definitely can't be a fixed size.
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(ValidationError)
    }

    // Could potentially be a fixed size, but fuck checking for that. It's not worth it. Future Rudy & Duncan can deal with it.
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(ValidationError)
    }

    // Fixed size. But we don't want to let them use Options.
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(ValidationError)
    }

    // Not a fixed size.
    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(ValidationError)
    }

    // Fixed size.
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    // This ones complicated. Technically we *can* make it so that it's a fixed size. However, we should decide on that at a later date.
    // TODO: Decide whether we want to allow them to use a byte array that can only be a max size.
    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(ValidationError)
    }

    // Fixed size.
    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    // Fixed size.
    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    // Fixed size.
    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    // Fixed size.
    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    // Fixed size.
    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
    // Fixed size.
    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    // Fixed size.
    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    // Fixed size.
    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    // Fixed size.
    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    // Fixed size.
    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    // Fixed size.
    fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
        Err(ValidationError)
    }

    // Fixed size.
    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    // Fixed size.
    fn serialize_i128(self, _v: i128) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    // Fixed size.
    fn serialize_u128(self, _v: u128) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    // Fixed size.
    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl SerializeSeq for FixedSizeValidator {
    type Ok = ();
    type Error = ValidationError;

    // Fixed size.
    fn serialize_element<T: ?Sized>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        Ok(())
    }

    // Fixed size.
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl SerializeTuple for FixedSizeValidator {
    type Ok = ();
    type Error = ValidationError;

    // Fixed size.
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(Self::new())
    }

    // Fixed size.
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl SerializeTupleStruct for FixedSizeValidator {
    type Ok = ();
    type Error = ValidationError;

    // Fixed size.
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(Self::new())
    }

    // Fixed size.
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl SerializeTupleVariant for FixedSizeValidator {
    type Ok = ();
    type Error = ValidationError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(Self::new())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl SerializeMap for FixedSizeValidator {
    type Ok = ();
    type Error = ValidationError;

    // Fixed size.
    fn serialize_key<T: ?Sized>(&mut self, _key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        Ok(())
    }

    // Fixed size.
    fn serialize_value<T: ?Sized>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        Ok(())
    }

    // Fixed size.
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl SerializeStruct for FixedSizeValidator {
    type Ok = ();
    type Error = ValidationError;

    // Fixed size.
    fn serialize_field<T: ?Sized>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(Self)
    }

    // Fixed size.
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl SerializeStructVariant for FixedSizeValidator {
    type Ok = ();
    type Error = ValidationError;

    // Fixed size.
    fn serialize_field<T: ?Sized>(
        &mut self,
        _key: &'static str,
        _value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        // TODO: Decide if we ever want to support serializing struct variants of *uniform* size.
        // If you do, then write this to check for uniform sizes within struct variants.
        // I personally don't want to program this right now, so I'm just going to leave it as a TODO.
        Ok(())
    }

    // Fixed size.
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
