use bincode::{deserialize, serialize, Error};
use bytes::BytesMut;
use serde::Serialize;
use tokio_util::codec::{Decoder, Encoder};

use crate::node::replication::message::Request;

#[derive(Default)]
pub struct BincodeCodec;

impl<I> Encoder<I> for BincodeCodec
where
    I: Serialize,
{
    type Error = Error;

    fn encode(&mut self, item: I, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.extend_from_slice(&serialize(&item)?);
        Ok(())
    }
}

impl Decoder for BincodeCodec {
    type Item = Request<'static, 'static>;

    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            Ok(None)
        } else {
            Ok(Some(deserialize(&src.split_to(src.len()))?))
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;
    use tokio_util::codec::{Decoder, Encoder};

    use super::BincodeCodec;
    use crate::node::replication::message::{PrimaryRequest, Request};

    #[test]
    fn test_encode_decode_valid_data() {
        let mut buf = BytesMut::default();

        let item = Request::Primary(PrimaryRequest::Ping);

        BincodeCodec.encode(&item, &mut buf).unwrap();

        assert_eq!(item, BincodeCodec.decode(&mut buf).unwrap().unwrap());
    }

    #[test]
    fn test_decode_invalid_data() {
        let mut buf = BytesMut::default();
        buf.extend_from_slice(b"test");

        BincodeCodec.decode(&mut buf).unwrap_err();
    }
}
