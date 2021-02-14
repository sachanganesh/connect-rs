use std::array::TryFromSliceError;
use std::convert::TryInto;
use std::error::Error;

const VERSION: u16 = 1;

/// Encountered when there is an issue constructing, serializing, or deserializing a [`ConnectDatagram`].
///
#[derive(Debug, Clone)]
pub enum DatagramError {
    /// Tried to construct a [`ConnectDatagram`] with an empty message body.
    EmptyMessage,

    /// Tried to construct a [`ConnectDatagram`] with a message body larger than 100MB.
    TooLargeMessage,

    /// Did not provide the complete byte-string necessary to deserialize the [`ConnectDatagram`].
    IncompleteBytes,

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
            DatagramError::IncompleteBytes => formatter.write_str("did not provide the complete byte-string necessary to deserialize the `ConnectDatagram`"),
            DatagramError::BytesParseFail(err) => std::fmt::Display::fmt(err, formatter),
        }
    }
}

/// A simple size-prefixed packet format containing a version tag, recipient tag, and message body.
///
/// The version tag is decided by the library version and used to maintain backwards
/// compatibility with previous datagram formats.
///
#[derive(Clone)]
pub struct ConnectDatagram {
    version: u16,
    recipient: u16,
    data: Option<Vec<u8>>,
}

impl ConnectDatagram {
    /// Creates a new [`ConnectDatagram`] based on an intended recipient and message body.
    ///
    /// The version tag is decided by the library version and used to maintain backwards
    /// compatibility with previous datagram formats.
    ///
    /// This will return a [EmptyMessage](`DatagramError::EmptyMessage`) error if the `data`
    /// parameter contains no bytes, or in other words, when there is no message body.
    ///
    /// This will return a [TooLargeMessage](`DatagramError::TooLargeMessage`) error if the `data`
    /// parameter contains a buffer size greater than 100,000,000 (bytes), or 100MB.
    ///
    pub fn new(recipient: u16, data: Vec<u8>) -> Result<Self, DatagramError> {
        if data.len() > 100_000_000 {
            Err(DatagramError::TooLargeMessage)
        } else if data.len() > 0 {
            Ok(Self {
                version: VERSION,
                recipient,
                data: Some(data),
            })
        } else {
            Err(DatagramError::EmptyMessage)
        }
    }

    /// Gets the version number of the datagram.
    ///
    pub fn version(&self) -> u16 {
        self.version
    }

    /// Gets the recipient of the datagram.
    ///
    pub fn recipient(&self) -> u16 {
        self.recipient
    }

    /// Gets the message body of the datagram.
    ///
    pub fn data(&self) -> Option<&Vec<u8>> {
        self.data.as_ref()
    }

    /// Takes ownership of the message body of the datagram.
    ///
    pub fn take_data(&mut self) -> Option<Vec<u8>> {
        self.data.take()
    }

    /// Calculates the size-prefixed serialized byte-size of the datagram.
    ///
    /// This will include the byte-size of the size-prefix.
    ///
    pub fn size(&self) -> usize {
        let data_len = if let Some(data) = self.data() {
            data.len()
        } else {
            0
        };

        8 + data_len
    }

    /// Constructs a serialized representation of the datagram contents.
    ///
    pub(crate) fn bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.size());

        bytes.extend(&self.version.to_be_bytes());
        bytes.extend(&self.recipient.to_be_bytes());

        if let Some(data) = self.data() {
            bytes.extend(data.as_slice());
        }

        return bytes;
    }

    /// Serializes the datagram.
    ///
    pub fn encode(self) -> Vec<u8> {
        let content_encoded = self.bytes();
        let size: u32 = (content_encoded.len()) as u32;

        let mut bytes = Vec::from(size.to_be_bytes());
        bytes.extend(content_encoded);

        return bytes;
    }

    /// Deserializes the datagram from a buffer.
    ///
    /// The buffer **should not** contain the size-prefix, and only contain the byte contents of the
    /// struct (version, recipient, and message body).
    ///
    pub fn decode(mut buffer: Vec<u8>) -> Result<Self, DatagramError> {
        if buffer.len() > 4 {
            let mem_size = std::mem::size_of::<u16>();
            let data = buffer.split_off(mem_size * 2);

            let (version_bytes, recipient_bytes) = buffer.split_at(mem_size);

            match version_bytes.try_into() {
                Ok(version_slice) => match recipient_bytes.try_into() {
                    Ok(recipient_slice) => {
                        let version = u16::from_be_bytes(version_slice);
                        let recipient = u16::from_be_bytes(recipient_slice);

                        Ok(Self {
                            version,
                            recipient,
                            data: Some(data),
                        })
                    }

                    Err(err) => Err(DatagramError::BytesParseFail(err)),
                },

                Err(err) => Err(DatagramError::BytesParseFail(err)),
            }
        } else {
            Err(DatagramError::IncompleteBytes)
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

        let sample = ConnectDatagram::new(1, data)?;
        assert_eq!(8 + 5, sample.encode().len());

        Ok(())
    }

    #[test]
    fn take_data() -> anyhow::Result<()> {
        let mut data = Vec::new();
        for _ in 0..5 {
            data.push(1);
        }

        let mut sample = ConnectDatagram::new(1, data)?;

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

        let sample = ConnectDatagram::new(1, data)?;
        let serialized_size = sample.size();
        assert_eq!(8 + 5, serialized_size);

        let mut payload = sample.encode();
        assert_eq!(serialized_size, payload.len());

        let payload = payload.split_off(std::mem::size_of::<u32>());
        let sample_back_res = ConnectDatagram::decode(payload);
        assert!(sample_back_res.is_ok());

        let sample_back = sample_back_res.unwrap();
        assert_eq!(sample_back.version(), 1);
        assert_eq!(sample_back.recipient(), 1);
        assert_eq!(sample_back.data().unwrap().len(), 5);

        Ok(())
    }
}
