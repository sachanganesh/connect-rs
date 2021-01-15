mod message;

pub use message::StitchMessage;
use protobuf::well_known_types::Any;
use protobuf::Message;

impl StitchMessage {
    // @todo make pub(crate)
    pub fn from_msg<T: Message>(msg: T) -> Self {
        let mut sm = Self::new();
        let payload = Any::pack(&msg).expect("Protobuf Message could not be packed into Any type");

        sm.set_payload(payload);
        return sm;
    }
}
