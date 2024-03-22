use bytes::Bytes;
use serde::de::IntoDeserializer;
use std::io::{BufRead, BufReader, Read};

use crate::utils::trim_bytes;

use super::error::{Error, Result};

/// A convenience method for deserialize some object from a reader.
pub fn from_reader<'de, R: Read, T: serde::de::Deserialize<'de>>(reader: R) -> Result<T> {
    let mut de =
        serde_xml_rs::Deserializer::new_from_reader(reader).non_contiguous_seq_elements(true);
    T::deserialize(&mut de).map_err(Into::into)
}

/// A convenience method for deserialize some object from a str.
pub fn from_str<'de, T: serde::de::Deserialize<'de>>(s: &'de str) -> Result<T> {
    from_reader(s.as_bytes())
}

/// A convenience method for deserialize some object from a string.
pub fn from_string<'de, T: serde::de::Deserialize<'de>>(s: String) -> Result<T> {
    from_reader(s.as_bytes())
}

/// A convenience method for deserialize some object from a [Bytes].
pub fn from_bytes<'de, T: serde::de::Deserialize<'de>>(s: &'de Bytes) -> Result<T> {
    from_reader(s.as_ref())
}

macro_rules! deserialize_type {
    ($deserialize:ident, $visit:ident) => {
        fn $deserialize<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
        {
            let tag = self.top_tag()?;
            let result = visitor.$visit(tag.content().parse()?);
            if result.is_ok() {
                self.close_tag()?;
            }
            result
        }
    };
}

macro_rules! custom_error {
    ($info:expr) => {
        Err(Error::Custom {
            field: $info.to_owned(),
        })
    };
}

#[derive(Debug, Clone, PartialEq)]
enum EventType {
    Statement,
    EmptyTag,
    Tag,
    TagClose,
    Comment,
}

#[derive(Debug, Clone)]
struct Event {
    pub type_: EventType,
    pub value: Vec<u8>,
    pub content: Vec<u8>,
}

impl Event {
    pub fn new(type_: EventType, value: Vec<u8>, content: Vec<u8>) -> Self {
        Self {
            type_,
            value,
            content,
        }
    }

    #[inline]
    fn is_tag(&self) -> bool {
        self.type_ == EventType::Tag
    }

    #[inline]
    fn tag<'d>(&'d self) -> std::borrow::Cow<'d, str> {
        String::from_utf8_lossy(&self.value)
    }

    #[inline]
    fn content(&self) -> std::borrow::Cow<'_, str> {
        String::from_utf8_lossy(&self.content)
    }
}

struct Deserializer<R: Read> {
    source: BufReader<R>,
    tags: Vec<Event>,
    next_tag_cache: Option<Event>,
    init: bool,
}

impl<R: Read> Deserializer<R> {
    pub fn new(r: R) -> Self {
        Self {
            source: BufReader::new(r),
            tags: vec![],
            next_tag_cache: None,
            init: false,
        }
    }

    #[inline]
    fn top_tag(&self) -> Result<&Event> {
        if let Some(tag) = self.tags.last() {
            Ok(tag)
        } else {
            custom_error!("error tag")
        }
    }

    fn next_tag(&mut self) -> Result<Event> {
        let tag = self.next_tag_cache.take();
        if let Some(tag) = tag {
            return Ok(tag);
        }
        loop {
            let event = self.next_event()?;
            match event.type_ {
                EventType::EmptyTag | EventType::TagClose | EventType::Tag => return Ok(event),
                _ => continue,
            }
        }
    }

    fn next_tag_ref(&mut self) -> Result<&Event> {
        if self.next_tag_cache.is_none() {
            self.next_tag_cache = Some(self.next_tag()?);
        }
        Ok(unsafe { self.next_tag_cache.as_ref().unwrap_unchecked() })
    }

    fn close_tag(&mut self) -> Result<()> {
        let next_tag = self.next_tag()?;
        let top_tag = self.top_tag()?;
        if !next_tag.is_tag() {
            if top_tag.value == next_tag.value {
                self.tags.pop();
            } else {
                return Err(Error::UnexpectedToken {
                    token: top_tag.tag().to_string(),
                    found: next_tag.tag().to_string(),
                });
            }
        } else {
            self.tags.push(next_tag);
            self.close_tag()?;
            self.close_tag()?;
        }
        Ok(())
    }

    fn next_event(&mut self) -> Result<Event> {
        if !self.init {
            let mut buf = vec![];
            self.source.read_until(b'<', &mut buf)?;
            self.init = true;
        }
        let mut buf = vec![];
        self.source.read_until(b'>', &mut buf)?;

        if buf.len() == 0 {
            return custom_error!("Incorrect XML syntax");
        }

        let data = if buf.ends_with(b"/>") {
            (EventType::EmptyTag, &buf[..buf.len() - 2])
        } else if buf.starts_with(b"/") {
            (EventType::TagClose, &buf[1..buf.len() - 1])
        } else if buf.starts_with(b"!--") {
            (EventType::Comment, &buf[..buf.len() - 1])
        } else if buf.starts_with(b"?xml") {
            (EventType::Statement, &buf[..buf.len() - 1])
        } else {
            let mut i = 0;
            for b in buf.iter() {
                i += 1;
                if *b == b' ' {
                    break;
                }
            }
            (EventType::Tag, &buf[..i - 1])
        };
        let mut content = vec![];
        self.source.read_until(b'<', &mut content)?;
        let i = if content.len() > 1 {
            content.len() - 1
        } else {
            0
        };
        let content = trim_bytes(&content[..i]).to_owned();
        let event = Event::new(data.0, data.1.to_owned(), content);
        return Ok(event);
    }
}

impl<'de, 'a, R: Read> serde::de::Deserializer<'de> for &'a mut Deserializer<R> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.close_tag()?;
        visitor.visit_unit()
    }

    deserialize_type!(deserialize_bool, visit_bool);
    deserialize_type!(deserialize_i8, visit_i8);
    deserialize_type!(deserialize_i16, visit_i16);
    deserialize_type!(deserialize_i32, visit_i32);
    deserialize_type!(deserialize_i64, visit_i64);
    deserialize_type!(deserialize_u8, visit_u8);
    deserialize_type!(deserialize_u16, visit_u16);
    deserialize_type!(deserialize_u32, visit_u32);
    deserialize_type!(deserialize_u64, visit_u64);
    deserialize_type!(deserialize_f32, visit_f32);
    deserialize_type!(deserialize_f64, visit_f64);
    deserialize_type!(deserialize_string, visit_string);

    fn deserialize_str<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let tag = self.top_tag()?;
        let result = visitor.visit_str(&tag.content());
        if result.is_ok() {
            self.close_tag()?;
        }
        result
    }

    fn deserialize_bytes<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let tag = self.top_tag()?;
        let result = visitor.visit_bytes(&tag.content);
        if result.is_ok() {
            self.close_tag()?;
        }
        result
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let tag = self.top_tag()?;
        let result = visitor.visit_byte_buf(tag.content.clone());
        if result.is_ok() {
            self.close_tag()?;
        }
        result
    }

    serde::forward_to_deserialize_any! {
        char
        map
        unit
        unit_struct
        newtype_struct
        tuple
        tuple_struct
        identifier
    }

    fn deserialize_option<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let tag = self.top_tag()?.clone();
        let s = SeqAccess {
            de: self,
            tag,
            is_over: false,
        };
        visitor.visit_seq(s)
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        if !self.init {
            loop {
                let event = self.next_tag()?;
                if event.type_ == EventType::Tag && event.value == name.as_bytes() {
                    self.tags.push(event);
                    break;
                }
            }
        }
        let map_value = visitor.visit_map(self)?;
        Ok(map_value)
    }

    fn deserialize_enum<V>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let tag = self.top_tag()?;
        let result = visitor.visit_enum(tag.content().into_deserializer());
        if result.is_ok() {
            self.close_tag()?;
        }
        result
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.close_tag()?;
        visitor.visit_unit()
    }
}

impl<'de, 'a, R: Read> serde::de::MapAccess<'de> for &'a mut Deserializer<R> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> std::result::Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        loop {
            let event = self.next_tag_ref()?;
            if event.is_tag() {
                let event = self.next_tag()?;
                let cs = event.clone();
                self.tags.push(event);
                return seed.deserialize(cs.tag().into_deserializer()).map(Some);
            } else {
                self.close_tag()?;
                return Ok(None);
            }
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut **self)
    }
}

struct SeqAccess<'a, R: Read> {
    de: &'a mut Deserializer<R>,
    tag: Event,
    is_over: bool,
}

impl<'de, 'a, R: Read> serde::de::SeqAccess<'de> for SeqAccess<'a, R> {
    type Error = Error;

    fn next_element_seed<T>(
        &mut self,
        seed: T,
    ) -> std::result::Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        if self.is_over {
            return Ok(None);
        };
        let result = seed.deserialize(&mut *self.de).map(Some);
        let next_tag = self.de.next_tag_ref()?;
        if next_tag.is_tag() && next_tag.value == self.tag.value {
            let next_tag = self.de.next_tag()?;
            self.de.tags.push(next_tag);
            self.is_over = false;
        } else {
            self.is_over = true;
        }
        result
    }
}
