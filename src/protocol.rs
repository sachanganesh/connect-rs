use std::array::TryFromSliceError;
use std::convert::TryInto;
use std::error::Error;

const VERSION: u16 = 1;

const VERSION_BYTE_SIZE: usize = 4;
const TAG_BYTE_SIZE: usize = 4;
const SIZE_PREFIX_BYTE_SIZE: usize = 4;

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

    /// Wraps a [`TryFromSliceError`] encountered when the version or recipient tags cannot be
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
    header: Vec<u8>,
    data: Vec<u8>,
}

impl ConnectDatagram {
    /// Creates a new [`ConnectDatagram`] based on an intended recipient and message body.
    ///
    /// The version identifier is decided by the library version and used to maintain backwards
    /// compatibility with previous datagram formats.
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

    pub fn with_tag(tag: u16, data: Vec<u8>) -> Result<Self, DatagramError> {
        if data.len() > 100_000_000 {
            Err(DatagramError::TooLargeMessage)
        } else if data.len() > 0 {
            let mut buffer: Vec<u8> = Vec::with_capacity(SIZE_PREFIX_BYTE_SIZE + VERSION_BYTE_SIZE + TAG_BYTE_SIZE);

            buffer.extend(&VERSION.to_be_bytes());
            buffer.extend(&tag.to_be_bytes());
            buffer.extend(data);

            Ok(Self {
                buffer,
            })
        } else {
            Err(DatagramError::EmptyMessage)
        }
    }

    /// Gets the version number of the datagram.
    ///
    pub fn version(&self) -> u16 {
        todo!()
    }

    /// Gets the recipient of the datagram.
    ///
    pub fn tag(&self) -> u16 {
        let start = SIZE_PREFIX_BYTE_SIZE + VERSION_BYTE_SIZE;
        let end = start + TAG_BYTE_SIZE;

        let buf = self.buffer[start..end].as_ref().try_into().expect("could not parse big-endian bytes into tag variable");

        u16::from_be_bytes(buf)
    }

    pub fn set_tag(&mut self, tag: u16) {
        todo!()
    }

    /// Gets the message body of the datagram.
    ///
    pub fn data(&self) -> Option<&Vec<u8>> {
        todo!()
    }

    pub fn set_data(&mut self, data: Vec<u8>) {
        todo!()
    }

    /// Calculates the size-prefixed serialized byte-size of the datagram.
    ///
    /// This will include the byte-size of the size-prefix.
    ///
    pub fn serialized_size(&self) -> usize {
        self.buffer.len()
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

    /// Deserializes the datagram from a buffer.
    ///
    /// The buffer **should not** contain the size-prefix, and only contain the byte contents of the
    /// struct (version, recipient, and message body).
    ///
    pub fn from_bytes(buffer: &[u8]) -> Result<Self, DatagramError> {
        if buffer.len() > 4 {
            let mem_size = std::mem::size_of::<u16>();
            let (buffer, data) = buffer.split_at(mem_size * 2);

            let (version_bytes, recipient_bytes) = buffer.split_at(mem_size);

            match version_bytes.try_into() {
                Ok(version_slice) => match recipient_bytes.try_into() {
                    Ok(recipient_slice) => {
                        let version = u16::from_be_bytes(version_slice);
                        let recipient = u16::from_be_bytes(recipient_slice);

                        Ok(Self {
                            version,
                            tag: recipient,
                            data: Some(data.to_vec()),
                        })
                    }

                    Err(err) => Err(DatagramError::BytesParseFail(err)),
                },

                Err(err) => Err(DatagramError::BytesParseFail(err)),
            }
        } else {
            Err(DatagramError::InsufficientBytes)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::protocol::ConnectDatagram;

    #[test]
    fn serialized_size() -> anyhow::Result<()> {
        let mut data = Vec::new();
        for _ in 0..5 {
            data.push(1);
        }
        assert_eq!(5, data.len());

        let sample = ConnectDatagram::with_tag(1, data)?;
        assert_eq!(8 + 5, sample.into_bytes().len());

        Ok(())
    }

    #[test]
    fn take_data() -> anyhow::Result<()> {
        let mut data = Vec::new();
        for _ in 0..5 {
            data.push(1);
        }

        let mut sample = ConnectDatagram::with_tag(1, data)?;

        let taken_data = sample.take_data().unwrap();
        assert!(sample.data().is_none());
        assert_eq!(5, taken_data.len());

        Ok(())
    }

    #[async_std::test]
    async fn encode_and_decode() -> anyhow::Result<()> {
        let mut data = Vec::new();
        for _ in 0..5 {
            data.push(1);
        }
        assert_eq!(5, data.len());

        let sample = ConnectDatagram::with_tag(1, data)?;
        let serialized_size = sample.serialized_size();
        assert_eq!(8 + 5, serialized_size);

        let mut payload = sample.into_bytes();
        assert_eq!(serialized_size, payload.len());

        let payload = payload.split_off(std::mem::size_of::<u32>());
        let sample_back_res = ConnectDatagram::from_bytes(payload.as_slice());
        assert!(sample_back_res.is_ok());

        let sample_back = sample_back_res.unwrap();
        assert_eq!(sample_back.version(), 1);
        assert_eq!(sample_back.tag(), 1);
        assert_eq!(sample_back.data().unwrap().len(), 5);

        Ok(())
    }
}
