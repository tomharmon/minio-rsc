use std::io::{BufWriter, Write};

use serde::Serialize;

use super::error::Error;

/// A convenience method for serializing some object to a buffer.
#[inline]
pub fn to_writer<W: Write, S: Serialize>(writer: W, value: &S) -> Result<(), Error> {
    value.serialize(&mut Serializer::new(writer))
}

/// A convenience method for serializing some object to a string.
pub fn to_string<S: Serialize>(value: &S) -> Result<String, Error> {
    let mut writer = Vec::new();
    to_writer(&mut writer, value)?;
    String::from_utf8(writer).map_err(Into::into)
}

macro_rules! unsupport_type {
    ($type_:expr) => {
        Error::UnsupportedOperation {
            operation: format!("Serializing {}", $type_),
        }
    };
}

macro_rules! serialize_num_attr {
    ($name:ident, $type_:tt) => {
        #[inline]
        fn $name(self, v: $type_) -> Result<Self::Ok, Self::Error> {
            self.serialize_str(&v.to_string())
        }
    };
}

struct Serializer<W>
where
    W: Write,
{
    writer: BufWriter<W>,
    tags: Vec<&'static str>,
}

#[allow(unused)]
impl<W> Serializer<W>
where
    W: Write,
{
    fn new(writer: W) -> Self {
        Self {
            writer: BufWriter::new(writer),
            tags: vec![],
        }
    }

    fn write_tag(&mut self) -> Result<(), Error> {
        if let Some(tag) = self.tags.last() {
            self.writer.write_fmt(format_args!("<{tag}>"))?;
            Ok(())
        } else {
            Err(Error::Custom {
                field: "serialize fail with empty tag".to_owned(),
            })
        }
    }

    fn write_close_tag(&mut self) -> Result<(), Error> {
        if let Some(tag) = self.tags.last() {
            self.writer.write_fmt(format_args!("</{tag}>"))?;
            Ok(())
        } else {
            Err(Error::Custom {
                field: "serialize fail with empty /tag".to_owned(),
            })
        }
    }
}

#[allow(unused)]
impl<'ser, W: Write> serde::ser::Serializer for &'ser mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;

    type SerializeTuple = Self;

    type SerializeTupleStruct = Self;

    type SerializeTupleVariant = Self;

    type SerializeMap = Self;

    type SerializeStruct = Self;

    type SerializeStructVariant = Self;

    // fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
    //     Err(unsupport_type!("bool"))
    // }

    serialize_num_attr!(serialize_bool, bool);
    serialize_num_attr!(serialize_i8, i8);
    serialize_num_attr!(serialize_i16, i16);
    serialize_num_attr!(serialize_i32, i32);
    serialize_num_attr!(serialize_i64, i64);
    serialize_num_attr!(serialize_u8, u8);
    serialize_num_attr!(serialize_u16, u16);
    serialize_num_attr!(serialize_u32, u32);
    serialize_num_attr!(serialize_u64, u64);
    serialize_num_attr!(serialize_f32, f32);
    serialize_num_attr!(serialize_f64, f64);
    serialize_num_attr!(serialize_char, char);

    #[inline]
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.serialize_bytes(v.as_bytes())
    }

    #[inline]
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.write_tag()?;
        self.writer.write(v)?;
        self.write_close_tag()?;
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        // skip none
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(self)
    }

    /// The type of `()` in Rust.
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        // skip
        Ok(())
    }

    /// `struct Unit` or `PhantomData<T>`. It represents a named value containing no data.
    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(unsupport_type!("unit_struct"))
    }

    /// For example the `E::A` and `E::B` in enum `E { A, B }`
    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    /// For example `struct Millimeters(u8)`.
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(self)
    }

    /// For example the `E::N` in enum `E { N(u8) }`
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        Err(unsupport_type!("newtype_variant"))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(self)
    }

    /// A statically sized heterogeneous sequence of values for which the length will
    /// be known at deserialization time without looking at the serialized data,
    /// for example `(u8,)` or `(String, u64, Vec<T>)` or `[u64; 10]`.
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(unsupport_type!("tuple"))
    }

    /// A named tuple, for example `struct Rgb(u8, u8, u8)`.
    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(unsupport_type!("tuple_struct"))
    }

    /// For example the `E::T` in `enum E { T(u8, u8) }`.
    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(unsupport_type!("tuple_variant"))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(unsupport_type!("map"))
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        if self.tags.len() == 0 {
            self.tags.push(name);
        }
        self.write_tag();
        Ok(self)
    }

    /// For example the `E::S` in `enum E { S { r: u8, g: u8, b: u8 } }`.
    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(unsupport_type!("struct_variant"))
    }
}

impl<'ser, W: Write> serde::ser::SerializeSeq for &'ser mut Serializer<W> {
    type Ok = ();

    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

#[allow(unused)]
impl<'ser, W: Write> serde::ser::SerializeMap for &'ser mut Serializer<W> {
    type Ok = ();

    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        Err(unsupport_type!("Map"))
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        Err(unsupport_type!("Map"))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Err(unsupport_type!("Map"))
    }
}

#[allow(unused)]
impl<'ser, W: Write> serde::ser::SerializeTuple for &'ser mut Serializer<W> {
    type Ok = ();

    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        Err(unsupport_type!("Tuple"))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Err(unsupport_type!("Tuple"))
    }
}

#[allow(unused)]
impl<'ser, W: Write> serde::ser::SerializeTupleStruct for &'ser mut Serializer<W> {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        Err(unsupport_type!("TupleStruct"))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Err(unsupport_type!("TupleStruct"))
    }
}

#[allow(unused)]
impl<'ser, W: Write> serde::ser::SerializeTupleVariant for &'ser mut Serializer<W> {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        Err(unsupport_type!("TupleVariant"))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Err(unsupport_type!("TupleVariant"))
    }
}

impl<'ser, W: Write> serde::ser::SerializeStruct for &'ser mut Serializer<W> {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        self.tags.push(key);
        value.serialize(&mut **self)?;
        self.tags.pop();
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.write_close_tag()
    }
}

#[allow(unused)]
impl<'ser, W: Write> serde::ser::SerializeStructVariant for &'ser mut Serializer<W> {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        Err(unsupport_type!("StructVariant"))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Err(unsupport_type!("StructVariant"))
    }
}
