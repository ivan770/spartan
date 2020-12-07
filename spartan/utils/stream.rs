use std::{
    io::{Cursor, Error, Read},
    pin::Pin,
    task::{Context, Poll},
};

use actix_web::web::BytesMut;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::Encoder;

#[derive(Default)]
pub struct TestStream<'a> {
    input: Option<&'a mut BytesMut>,
    output: Cursor<BytesMut>,
}

impl<'a> TestStream<'a> {
    pub fn from_output<I, C>(item: I, encoder: &mut C) -> Result<Self, <C as Encoder<I>>::Error>
    where
        C: Encoder<I>,
    {
        let mut buf = BytesMut::new();
        encoder.encode(item, &mut buf)?;
        Ok(TestStream {
            input: None,
            output: Cursor::new(buf),
        })
    }

    pub fn input(mut self, buf: &'a mut BytesMut) -> Self {
        self.input.replace(buf);
        self
    }
}

impl AsyncWrite for TestStream<'_> {
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        if let Some(input) = self.get_mut().input.as_mut() {
            input.extend_from_slice(buf);
        }

        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Poll::Ready(Ok(()))
    }
}

impl AsyncRead for TestStream<'_> {
    fn poll_read(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        Poll::Ready(self.get_mut().output.read(buf))
    }
}

#[cfg(test)]
mod tests {
    use actix_web::web::BytesMut;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio_util::codec::{Decoder, LinesCodec};

    use super::TestStream;

    #[tokio::test]
    async fn test_input() {
        let mut buf = BytesMut::default();
        let mut stream = TestStream::default().input(&mut buf);
        stream.write_all(b"Hello, world").await.unwrap();
        assert_eq!(&*buf, b"Hello, world");
    }

    #[tokio::test]
    async fn test_output() {
        let mut codec = LinesCodec::default();
        let mut stream = TestStream::from_output("123", &mut codec).unwrap();
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await.unwrap();
        assert_eq!(
            codec.decode(&mut BytesMut::from(&*buf)).unwrap().unwrap(),
            "123"
        );
    }
}
