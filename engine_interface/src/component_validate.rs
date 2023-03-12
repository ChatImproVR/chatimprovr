use crate::component_validate_error::Error as CompilerHappinessError;
use serde;
use serde::Serialize;
use serde::ser::{SerializeSeq, SerializeTuple, SerializeTupleStruct, SerializeTupleVariant, SerializeStructVariant, SerializeStruct, SerializeMap};
pub struct CustomSerializer;

impl serde::Serializer for CustomSerializer {
    type Ok = ();
    type Error = CompilerHappinessError;
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
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(Self)
    }
    // Variable size
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(todo!())
    }
    // Variable size
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(todo!())
    }
    // Fixed size
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(Self)
    }
    // Fixed size, legit doesn't contain data.
    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
    // Fixed size, idfk man. These things dont hold data.
    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
    // This is a fixed size. It's just a wrapper around a single value. The value is what matters and can be variable size.
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Ok(())
    }
    // Fixed size. It's a wrapper around multiple values, which could potentially have a size.
    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(Self)
    }
    // Not a fixed size. We're not getting into fuckery with enums that have variants that could POTENTIALLY have data. Sugma.
    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(todo!())
    }

    // Definitely can't be a fixed size.
    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(todo!())
    }

    // Could potentially be a fixed size, but fuck checking for that. It's not worth it. Future Rudy & Duncan can deal with it.
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Ok(todo!())
    }

    // Fixed size. But we don't want to let them use Options.
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(todo!())
    }

    // Not a fixed size.
    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Ok(todo!())
    }

    // Fixed size.
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    // This ones complicated. Technically we *can* make it so that it's a fixed size. However, we should decide on that at a later date.
    // TODO: Decide whether we want to allow them to use a byte array that can only be a max size.
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(todo!())
    }

    // Fixed size.
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
    
    // Fixed size.
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
    
    // Fixed size.
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    // Fixed size.
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    // Fixed size.
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
    // Fixed size.
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    // Fixed size.
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    // Fixed size.
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    // Fixed size.
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    // Fixed size.
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    // Fixed size.
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    // Fixed size.
    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    // Fixed size.
    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    // Fixed size.
    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    // Fixed size.
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl SerializeSeq for CustomSerializer{

    type Ok = ();
    type Error = CompilerHappinessError;

    // Fixed size.
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
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

impl SerializeTuple for CustomSerializer{

    type Ok = ();
    type Error = CompilerHappinessError;

    // Fixed size.
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
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

impl SerializeTupleStruct for CustomSerializer{

    type Ok = ();
    type Error = CompilerHappinessError;

    // Fixed size.
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
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

impl SerializeTupleVariant for CustomSerializer{

    type Ok = ();
    type Error = CompilerHappinessError;

    // Fixed size.
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
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

impl SerializeMap for CustomSerializer{

    type Ok = ();
    type Error = CompilerHappinessError;

    // Fixed size.
    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        Ok(())
    }

    // Fixed size.
    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
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

impl SerializeStruct for CustomSerializer{

    type Ok = ();
    type Error = CompilerHappinessError;

    // Fixed size.
    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
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

impl SerializeStructVariant for CustomSerializer{

    type Ok = ();
    type Error = CompilerHappinessError;

    // Fixed size.
    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
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

