use std::{collections::HashMap, ops::Range, pin::Pin};

use async_stream::stream as Stream2;
use bytes::{Bytes, BytesMut};
use futures_core::Stream;
use futures_util::StreamExt;

use crate::{datatype::OutputSerialization, error::{Error, Result}};

/// read u32 from `&[u8]`
/// # Panics
/// Panics if `data.len() != 4`.
#[inline]
fn read_u32(data: &[u8]) -> u32 {
    u32::from_be_bytes(<[u8; 4]>::try_from(data).unwrap())
}

/// read u16 from `&[u8]`
/// # Panics
/// Panics if `data.len() != 2`.
#[inline]
fn read_u16(data: &[u8]) -> u16 {
    u16::from_be_bytes(<[u8; 2]>::try_from(data).unwrap())
}

/// the event type of message from select object content.
#[derive(PartialEq, Eq)]
enum EventType {
    Records,
    Continuation,
    Progress,
    Stats,
    End,
    RequestLevelError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum HeaderName {
    Messagetype,
    EventType,
    ErrorCode,
    ErrorMessage,
}

/// message from select object content.
pub struct Message {
    data: Bytes,
    type_: EventType,
    payload: Range<usize>,
    headers: HashMap<HeaderName, String>,
}

impl<'a> Message {
    pub fn payload(&self) -> &[u8] {
        &self.data[self.payload.clone()]
    }

    /// Message type is Records. It can contain a single record, a partial record, or multiple records, depending on the number of search results.
    pub fn is_records(&self) -> bool {
        self.type_ == EventType::Records
    }

    /// Message type is Progress.
    pub fn is_progress(&self) -> bool {
        self.type_ == EventType::Progress
    }

    /// Message type is Stats.
    pub fn is_stats(&self) -> bool {
        self.type_ == EventType::Stats
    }

    /// Message type is Continuation.
    pub fn is_continuation(&self) -> bool {
        self.type_ == EventType::Continuation
    }

    /// Message type is End.
    pub fn is_end(&self) -> bool {
        self.type_ == EventType::End
    }

    /// return the value of *:message-type* header.
    pub fn message_type(&self) -> Option<&String> {
        self.headers.get(&HeaderName::Messagetype)
    }

    /// Message type is Error, more info by `error_code` `error_message` method.
    /// If returns this information, the End message information will not be returned.
    pub fn is_error(&self) -> bool {
        self.type_ == EventType::RequestLevelError
    }

    /// return the value of *:error-code* header, None if this Message is not error.
    pub fn error_code(&self) -> Option<&String> {
        self.headers.get(&HeaderName::ErrorCode)
    }

    /// return the value of *:error-message* header, None if this Message is not error.
    pub fn error_message(&self) -> Option<&String> {
        self.headers.get(&HeaderName::ErrorMessage)
    }
}

impl<'a> TryFrom<Bytes> for Message {
    type Error = String;

    fn try_from(data: Bytes) -> std::result::Result<Self, Self::Error> {
        let prelude_crc = read_u32(&data[8..12]);
        let prelude_crc_calc = crc32fast::hash(&data[0..8]);
        if prelude_crc != prelude_crc_calc {
            return Err(format!(
                "prelude CRC mismatch; expected: {prelude_crc}, got: {prelude_crc_calc}"
            ));
        }
        let message_crc = read_u32(&data[data.len() - 4..]);
        let message_crc_calc = crc32fast::hash(&data[0..data.len() - 4]);
        if message_crc != message_crc_calc {
            return Err(format!(
                "message CRC mismatch; expected: {message_crc}, got: {message_crc_calc}"
            ));
        }
        let header_length = read_u32(&data[4..8]) as usize;
        let header_end = 12 + header_length;

        let payload = 12 + header_length..data.len() - 4;

        let mut pos = 12;
        let mut headers = HashMap::new();
        loop {
            let key_len = data[pos] as usize;
            pos += 1;
            let key = &data[pos..pos + key_len];
            pos += key_len + 1;
            let value_len = read_u16(&data[pos..pos + 2]) as usize;
            pos += 2;
            let val = &data[pos..pos + value_len];
            let val = String::from_utf8(val.to_vec()).unwrap();
            pos += value_len;
            let header_name = match key {
                b":message-type" => HeaderName::Messagetype,
                b":event-type" => HeaderName::EventType,
                b":error-code" => HeaderName::ErrorCode,
                b":error-message" => HeaderName::ErrorMessage,
                _ => continue,
            };
            headers.insert(header_name, val);
            if pos >= header_end {
                break;
            }
        }
        if let Some(event_type) = headers.get(&HeaderName::EventType) {
            let type_: EventType = match event_type.as_str() {
                "Continuation" => EventType::Continuation,
                "Progress" => EventType::Progress,
                "Records" => EventType::Records,
                "Stats" => EventType::Stats,
                "End" => EventType::End,
                ev => return Err(format!("unknown event type: {ev:?}")),
            };
            return Ok(Message {
                data,
                type_,
                payload,
                headers,
            });
        } else {
            if headers.contains_key(&HeaderName::ErrorCode) {
                return Ok(Message {
                    data,
                    type_: EventType::RequestLevelError,
                    payload,
                    headers,
                });
            } else {
                Err(format!("unknown message"))
            }
        }
    }
}

/// reader response data of `select_object_content` method
pub struct SelectObjectReader {
    response: reqwest::Response,
    output_serialization: OutputSerialization,
}

impl SelectObjectReader {
    pub(crate) fn new(
        response: reqwest::Response,
        output_serialization: OutputSerialization,
    ) -> Self {
        Self {
            response,
            output_serialization,
        }
    }

    /// Read [Message] as streams
    pub fn read_message(mut self) -> Pin<Box<dyn Stream<Item = Result<Message>> + Send>> {
        Box::pin(Stream2! {
            let mut buf = BytesMut::new();
            let mut msg_len = 0;
            let mut is_over = false;
            loop{
                if !is_over{
                    if let Some(data) = self.response.chunk().await?{
                        buf.extend_from_slice(&data);
                    }else{
                        is_over = true;
                    };
                }else{
                    match buf.len(){
                        0=>break,
                        l if l < 4 => Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, format!("not enough data in the stream; expected: 4, got: {} bytes", l)))?,
                        l if l < msg_len => Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, format!("not enough data in the stream; expected: {}, got: {} bytes", msg_len, l)))?,
                        _=>{}
                    }
                }
                if msg_len == 0 && buf.len() >= 4{
                    msg_len = read_u32(&buf[0..4]) as usize;
                }
                if msg_len > 0 && buf.len() >= msg_len{
                    let msg_data = buf.split_to(msg_len);
                    msg_len = 0;
                    yield Ok(Message::try_from(msg_data.freeze()).map_err(|e| Error::MessageDecodeError(e))?);
                }
            }
        })
    }

    /// Read all response data at once and decode the content to bytes.
    pub async fn read_all(self) -> Result<Bytes> {
        let mut data = BytesMut::new();
        let mut messages = self.read_message();
        while let Some(message) = messages.next().await {
            let message = message?;
            if message.is_records() {
                data.extend_from_slice(message.payload());
            } else if message.is_error() {
                Err(Error::SelectObejectError(format!(
                    "Select Message Error code: {:?}, error message: {:?}",
                    message.error_code(),
                    message.error_message(),
                )))?
            }
        }
        Ok(data.freeze())
    }

    /// get [OutputSerialization]
    pub fn output_serialization(&self) -> &OutputSerialization {
        &self.output_serialization
    }
}
