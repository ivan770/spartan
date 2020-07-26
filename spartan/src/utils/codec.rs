use crate::node::replication::message::Request;
use actix_web::web::BytesMut;
use bincode::{deserialize, serialize, Error};
use serde::Serialize;
use tokio_util::codec::{Decoder, Encoder};

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
    // GAT's required to have generic impl
    type Item = Request<'static>;

    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if !src.is_empty() {
            Ok(Some(deserialize(&src.split_to(src.len()))?))
        } else {
            Ok(None)
        }
    }
}
