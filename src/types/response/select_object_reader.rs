use std::ops::Range;

use bytes::{Bytes, BytesMut};

use crate::errors::{Result, ValueError};

/// read u32 from `&[u8]`
#[inline]
fn read_u32(data: &[u8]) -> std::result::Result<u32, std::array::TryFromSliceError> {
    Ok(u32::from_be_bytes(<[u8; 4]>::try_from(data)?))
}

/// read u16 from `&[u8]`
#[inline]
fn read_u16(data: &[u8]) -> std::result::Result<u16, std::array::TryFromSliceError> {
    Ok(u16::from_be_bytes(<[u8; 2]>::try_from(data)?))
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

/// message from select object content.
pub struct Message {
    data: Bytes,
    type_: EventType,
    payload: Range<usize>,
}

impl Message {
    pub fn payload(&self) -> &[u8] {
        &self.data[self.payload.clone()]
    }

    pub fn is_records(&self) -> bool {
        self.type_ == EventType::Records
    }

    pub fn is_error(&self) -> bool {
        self.type_ == EventType::RequestLevelError
    }
}

impl TryFrom<Bytes> for Message {
    type Error = String;

    fn try_from(data: Bytes) -> std::result::Result<Self, Self::Error> {
        let prelude_crc =
            read_u32(&data[8..12]).map_err(|_| "fail to read prelude crc".to_owned())?;
        let prelude_crc_calc = crc32fast::hash(&data[0..8]);
        if prelude_crc != prelude_crc_calc {
            return Err(format!(
                "prelude CRC mismatch; expected: {prelude_crc}, got: {prelude_crc_calc}"
            ));
        }
        let message_crc =
            read_u32(&data[data.len() - 4..]).map_err(|_| "fail to read message crc".to_owned())?;
        let message_crc_calc = crc32fast::hash(&data[0..data.len() - 4]);
        if message_crc != message_crc_calc {
            return Err(format!(
                "message CRC mismatch; expected: {message_crc}, got: {message_crc_calc}"
            ));
        }
        let header_length = read_u32(&data[4..8])
            .map_err(|_| "fail to read header byte length".to_owned())?
            as usize;
        let header_end = 12 + header_length;

        let payload = 12 + header_length..data.len() - 4;

        let mut pos = 12;
        loop {
            let key_len = data[pos] as usize;
            pos += 1;
            let key = pos..pos + key_len;
            pos += key_len + 1;
            let value_len = read_u16(&data[pos..pos + 2])
                .map_err(|_| "fail to read header value length".to_owned())?
                as usize;
            pos += 2;
            let val = pos..pos + value_len;
            pos += value_len;
            if &data[key.clone()] == b":event-type" {
                let type_ = match &data[val] {
                    b"Continuation" => EventType::Continuation,
                    b"Progress" => EventType::Progress,
                    b"Records" => EventType::Records,
                    b"Stats" => EventType::Stats,
                    b"End" => EventType::End,
                    ev => return Err(format!("unknown event type: {ev:?}")),
                };
                return Ok(Message {
                    data,
                    type_,
                    payload,
                });
            }
            if &data[key] == b":error-code" {
                return Ok(Message {
                    data,
                    type_: EventType::RequestLevelError,
                    payload,
                });
            }
            if pos >= header_end {
                break;
            }
        }
        return Err("invalid message body".to_owned());
    }
}

/// reader response data of `select_object_content` method
#[derive(Debug)]
pub struct SelectObjectReader {
    response: reqwest::Response,
}

impl SelectObjectReader {
    pub(crate) fn new(response: reqwest::Response) -> Self {
        Self { response }
    }

    /// Read all response data at once and decode the content to bytes.
    pub async fn read_all(self) -> Result<Bytes> {
        let res = self.response.bytes().await?;
        let mut pos = 0;
        let mut da = BytesMut::new();
        loop {
            let total_length = read_u32(&res[pos..pos + 4])
                .map_err(|_| ValueError::new("fail to read total byte length"))?;
            let end_pos = pos + total_length as usize;
            if end_pos > res.len() {
                Err(ValueError::new("error total byte length is over"))?
            }
            let data = res.slice(pos..end_pos);
            let message = Message::try_from(data).map_err(|e| ValueError::new(e))?;
            if message.is_records() {
                da.extend_from_slice(message.payload());
            } else if message.is_error() {
                Err(ValueError::new("Select Error"))?
            }
            pos = end_pos;
            if pos < res.len() {
                continue;
            }
            break;
        }
        Ok(da.freeze())
    }
}
