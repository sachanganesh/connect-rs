use std::error::Error;
use std::io::Read;

const VERSION: u8 = 1;

/// Encountered when trying to construct a [`ConnectDatagram`] with an empty message body.
///
#[derive(Debug, Clone)]
pub struct DatagramEmptyError;

impl Error for DatagramEmptyError {}

impl std::fmt::Display for DatagramEmptyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "datagram cannot be constructed when provided payload is empty"
        )
    }
}

/// A simple packet format containing a version, recipient tag, and message body.
///
pub struct ConnectDatagram {
    version: u8,
    recipient: u16,
    data: Option<Vec<u8>>,
}

impl ConnectDatagram {
    /// Creates a new [`ConnectDatagram`] based on an intended recipient and message body.
    ///
    /// This will return a [`DatagramEmptyError`] if the `data` parameter contains no bytes, or
    /// in other words, when there is no message body.
    ///
    /// The version field is decided by the library version and used to maintain backwards
    /// compatibility with previous datagram formats.
    ///
    pub fn new(recipient: u16, data: Vec<u8>) -> Result<Self, DatagramEmptyError> {
        if data.len() > 0 {
            Ok(Self {
                version: VERSION,
                recipient,
                data: Some(data),
            })
        } else {
            Err(DatagramEmptyError)
        }
    }

    /// Gets the version number of the datagram.
    ///
    pub fn version(&self) -> u8 {
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

    /// Calculates the serialized byte-size of the datagram.
    ///
    pub fn size(&self) -> usize {
        let data_len = if let Some(data) = self.data() {
            data.len()
        } else {
            0
        };

        3 + data_len
    }

    /// Constructs a serialized representation of the datagram.
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
    pub fn encode(&self) -> Vec<u8> {
        let size: u32 = (self.size()) as u32;

        let mut bytes = Vec::from(size.to_be_bytes());
        bytes.extend(self.bytes());

        return bytes;
    }

    /// Deserializes the datagram from a `source`.
    ///
    pub fn decode(source: &mut (dyn Read + Send + Sync)) -> anyhow::Result<Self> {
        // payload size
        let mut payload_size_bytes: [u8; 4] = [0; 4];
        source.read_exact(&mut payload_size_bytes)?;
        let payload_size = u32::from_be_bytes(payload_size_bytes);

        // read whole payload
        let mut payload_bytes = vec![0; payload_size as usize];
        source.read_exact(payload_bytes.as_mut_slice())?;

        // version
        let version_bytes = payload_bytes.remove(0);
        let version = u8::from_be(version_bytes);

        // recipient
        let mut recipient_bytes: [u8; 2] = [0; 2];
        for i in 0..recipient_bytes.len() {
            recipient_bytes[i] = payload_bytes.remove(0);
        }
        let recipient = u16::from_be_bytes(recipient_bytes);

        // data
        let data = payload_bytes;

        if data.len() > 0 {
            Ok(Self {
                version,
                recipient,
                data: Some(data),
            })
        } else {
            Err(anyhow::Error::from(DatagramEmptyError))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::protocol::ConnectDatagram;
    use std::io::Cursor;

    #[test]
    fn encoded_size() -> anyhow::Result<()> {
        let mut data = Vec::new();
        for _ in 0..5 {
            data.push(1);
        }
        assert_eq!(5, data.len());

        let sample = ConnectDatagram::new(1, data)?;
        assert_eq!(7 + 5, sample.encode().len());

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

        let mut payload = sample.encode();
        assert_eq!(7 + 5, payload.len());

        let mut cursor: Cursor<&mut [u8]> = Cursor::new(payload.as_mut());
        let sample_back_res = ConnectDatagram::decode(&mut cursor);
        assert!(sample_back_res.is_ok());

        let sample_back = sample_back_res.unwrap();
        assert_eq!(sample_back.version(), 1);
        assert_eq!(sample_back.recipient(), 1);
        assert_eq!(sample_back.data().unwrap().len(), 5);

        Ok(())
    }
}
