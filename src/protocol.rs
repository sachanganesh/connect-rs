use std::array::TryFromSliceError;
use std::convert::TryInto;
use std::error::Error;

const VERSION: u16 = 1;

pub const SIZE_PREFIX_BYTE_SIZE: usize = 4;
const VERSION_BYTE_SIZE: usize = 2;
const TAG_BYTE_SIZE: usize = 2;

pub const DATAGRAM_HEADER_BYTE_SIZE: usize =
    SIZE_PREFIX_BYTE_SIZE + VERSION_BYTE_SIZE + TAG_BYTE_SIZE;

/// Encountered when there is an issue constructing, serializing, or deserializing a [`ConnectDatagram`].
///
#[derive(Debug, Clone)]
pub enum DatagramError {
    /// Tried to construct a [`ConnectDatagram`] with an empty message body.
    EmptyMessage,

    /// Tried to construct a [`ConnectDatagram`] with a message body larger than 100MB.
    TooLargeMessage,

    /// Did not provide the complete byte-string necessary to deserialize the [`ConnectDatagram`].
    InsufficientBytes,

    /// Wraps a [`TryFromSliceError`] encountered when the version or tag fields cannot be
    /// parsed from the provided bytes.
    BytesParseFail(TryFromSliceError),
}

impl Error for DatagramError {}

impl std::fmt::Display for DatagramError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DatagramError::EmptyMessage => formatter.write_str("tried to construct a `ConnectDatagram` with an empty message body"),
            DatagramError::TooLargeMessage => formatter.write_str("tried to construct a `ConnectDatagram` with a message body larger than 100MB"),
            DatagramError::InsufficientBytes => formatter.write_str("did not provide the complete byte-string necessary to deserialize the `ConnectDatagram`"),
            DatagramError::BytesParseFail(err) => std::fmt::Display::fmt(err, formatter),
        }
    }
}

/// A simple size-prefixed packet format containing a version id, optional tag, and message payload.
///
/// The version tag is decided by the library version and used to maintain backwards
/// compatibility with previous datagram formats.
///
#[derive(Clone)]
pub struct ConnectDatagram {
    buffer: Vec<u8>,
}

#[allow(dead_code)]
impl ConnectDatagram {
    /// Creates a new [`ConnectDatagram`] with the intended message body.
    ///
    /// This will return a [EmptyMessage](`DatagramError::EmptyMessage`) error if the `data`
    /// parameter contains no bytes, or in other words, when there is no message body.
    ///
    /// This will return a [TooLargeMessage](`DatagramError::TooLargeMessage`) error if the `data`
    /// parameter contains a buffer size greater than 100,000,000 (bytes), or 100MB.
    ///
    pub fn new(data: Vec<u8>) -> Result<Self, DatagramError> {
        Self::with_tag(0, data)
    }

    /// Creates a new [`ConnectDatagram`] based on an intended tag field and message body.
    ///
    /// This will return a [EmptyMessage](`DatagramError::EmptyMessage`) error if the `data`
    /// parameter contains no bytes, or in other words, when there is no message body.
    ///
    /// This will return a [TooLargeMessage](`DatagramError::TooLargeMessage`) error if the `data`
    /// parameter contains a buffer size greater than 100,000,000 (bytes), or 100MB.
    ///
    pub fn with_tag(tag: u16, data: Vec<u8>) -> Result<Self, DatagramError> {
        if data.len() > 100_000_000 {
            Err(DatagramError::TooLargeMessage)
        } else if data.len() > 0 {
            let mut buffer: Vec<u8> = Vec::with_capacity(DATAGRAM_HEADER_BYTE_SIZE + data.len());

            buffer.extend(
                ((DATAGRAM_HEADER_BYTE_SIZE - SIZE_PREFIX_BYTE_SIZE + data.len()) as u32)
                    .to_be_bytes(),
            );
            buffer.extend(VERSION.to_be_bytes());
            buffer.extend(tag.to_be_bytes());
            buffer.extend(data);

            Ok(Self { buffer })
        } else {
            Err(DatagramError::EmptyMessage)
        }
    }

    /// Updates the size prefix value in the internal buffer to the current size of the buffer.
    ///
    #[inline]
    fn update_size_prefix(&mut self) {
        self.buffer.splice(
            ..VERSION_BYTE_SIZE,
            ((DATAGRAM_HEADER_BYTE_SIZE - SIZE_PREFIX_BYTE_SIZE + self.data_size()) as u32)
                .to_be_bytes(),
        );
    }

    /// Gets the version number field of the datagram protocol.
    ///
    pub fn version(&self) -> u16 {
        let start = SIZE_PREFIX_BYTE_SIZE;
        let end = start + VERSION_BYTE_SIZE;

        let buf = self.buffer[start..end]
            .as_ref()
            .try_into()
            .expect("could not parse big-endian bytes into version variable");

        u16::from_be_bytes(buf)
    }

    /// Gets the tag field of the datagram.
    ///
    pub fn tag(&self) -> u16 {
        let start = SIZE_PREFIX_BYTE_SIZE + VERSION_BYTE_SIZE;
        let end = start + TAG_BYTE_SIZE;

        let buf = self.buffer[start..end]
            .as_ref()
            .try_into()
            .expect("could not parse big-endian bytes into tag variable");

        u16::from_be_bytes(buf)
    }

    /// Sets the message body of the datagram.
    ///
    pub fn set_tag(&mut self, tag: u16) {
        let start = SIZE_PREFIX_BYTE_SIZE + VERSION_BYTE_SIZE;
        let end = start + TAG_BYTE_SIZE;

        self.buffer.splice(start..end, tag.to_be_bytes());
    }

    /// Gets the message body of the datagram.
    ///
    pub fn data(&self) -> &[u8] {
        &self.buffer[DATAGRAM_HEADER_BYTE_SIZE..]
    }

    /// Sets the message body of the datagram and returns the previous contents.
    ///
    pub fn set_data(&mut self, data: Vec<u8>) -> Result<Vec<u8>, DatagramError> {
        let data_size = data.len();

        if data_size > 100_000_000 {
            Err(DatagramError::TooLargeMessage)
        } else if data_size > 0 {
            if data_size < self.buffer.len() {
                self.buffer.truncate(DATAGRAM_HEADER_BYTE_SIZE + data_size);
            }

            let old_data = self
                .buffer
                .splice(DATAGRAM_HEADER_BYTE_SIZE.., data)
                .collect();

            self.update_size_prefix();

            Ok(old_data)
        } else {
            Err(DatagramError::EmptyMessage)
        }
    }

    /// Calculates the size-prefixed serialized byte-size of the datagram.
    ///
    /// This will include the byte-size of the size-prefix.
    ///
    pub fn serialized_size(&self) -> usize {
        self.buffer.len()
    }

    /// Calculates the byte-size of the datagram message body.
    ///
    /// This will exclude all datagram header fields like the tag.
    ///
    pub fn data_size(&self) -> usize {
        self.buffer.len() - DATAGRAM_HEADER_BYTE_SIZE
    }

    /// Constructs a serialized representation of the datagram contents.
    ///
    pub(crate) fn as_bytes(&self) -> &[u8] {
        self.buffer.as_slice()
    }

    /// Serializes the datagram.
    ///
    pub fn into_bytes(self) -> Vec<u8> {
        self.buffer
    }

    /// Deserializes the datagram from bytes.
    ///
    pub fn from_bytes(buffer: &[u8]) -> Result<Self, DatagramError> {
        if buffer.len() > DATAGRAM_HEADER_BYTE_SIZE {
            Ok(Self {
                buffer: buffer.to_vec(),
            })
        } else {
            Err(DatagramError::InsufficientBytes)
        }
    }

    /// Deserializes the datagram from bytes, and infers the size-prefix given the data.
    ///
    pub fn from_bytes_without_prefix(buffer: &[u8]) -> Result<Self, DatagramError> {
        if buffer.len() > DATAGRAM_HEADER_BYTE_SIZE - SIZE_PREFIX_BYTE_SIZE {
            let mut new_buffer = Vec::with_capacity(SIZE_PREFIX_BYTE_SIZE + buffer.len());
            new_buffer.extend((buffer.len() as u32).to_be_bytes());
            new_buffer.extend_from_slice(buffer);

            Ok(Self { buffer: new_buffer })
        } else {
            Err(DatagramError::InsufficientBytes)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{protocol::ConnectDatagram, DATAGRAM_HEADER_BYTE_SIZE};

    #[test]
    fn serialized_size() -> anyhow::Result<()> {
        let data: Vec<u8> = vec![0, 1, 2, 3, 4];
        assert_eq!(5, data.len());

        let sample = ConnectDatagram::with_tag(1, data)?;
        assert_eq!(DATAGRAM_HEADER_BYTE_SIZE + 5, sample.serialized_size());
        assert_eq!(DATAGRAM_HEADER_BYTE_SIZE + 5, sample.into_bytes().len());

        Ok(())
    }

    #[test]
    fn get_data() -> anyhow::Result<()> {
        let data: Vec<u8> = vec![0, 1, 2, 3, 4];
        assert_eq!(5, data.len());

        let sample = ConnectDatagram::with_tag(1, data)?;

        let taken_data = sample.data();
        assert_eq!(5, taken_data.len());

        Ok(())
    }

    #[test]
    fn encode_and_decode() -> anyhow::Result<()> {
        let data: Vec<u8> = vec![0, 1, 2, 3, 4];
        assert_eq!(5, data.len());

        let sample = ConnectDatagram::with_tag(1, data)?;
        let serialized_size = sample.serialized_size();
        assert_eq!(DATAGRAM_HEADER_BYTE_SIZE + 5, serialized_size);

        let payload = sample.into_bytes();
        assert_eq!(serialized_size, payload.len());

        let sample_back_res = ConnectDatagram::from_bytes(payload.as_slice());
        assert!(sample_back_res.is_ok());

        let sample_back = sample_back_res.unwrap();
        assert_eq!(sample_back.version(), 1);
        assert_eq!(sample_back.tag(), 1);
        assert_eq!(sample_back.data().len(), 5);

        Ok(())
    }

    #[test]
    fn encode_and_decode_without_prefix() -> anyhow::Result<()> {
        let data: Vec<u8> = vec![0, 1, 2, 3, 4];
        assert_eq!(5, data.len());

        let sample = ConnectDatagram::with_tag(1, data)?;
        let serialized_size = sample.serialized_size();
        assert_eq!(DATAGRAM_HEADER_BYTE_SIZE + 5, serialized_size);

        let mut payload = sample.into_bytes();
        assert_eq!(serialized_size, payload.len());

        let payload = payload.split_off(crate::protocol::SIZE_PREFIX_BYTE_SIZE);
        let sample_back_res = ConnectDatagram::from_bytes_without_prefix(payload.as_slice());
        assert!(sample_back_res.is_ok());

        let sample_back = sample_back_res.unwrap();
        assert_eq!(sample_back.version(), 1);
        assert_eq!(sample_back.tag(), 1);
        assert_eq!(sample_back.data().len(), 5);

        Ok(())
    }
}
