use actix_web::web::BytesMut;
use std::{
    io::{Cursor, Error, Read},
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::Encoder;

#[derive(Default)]
pub struct TestStream {
    input: BytesMut,
    output: Cursor<BytesMut>,
}

impl TestStream {
    pub fn with_output<I, C>(item: I, encoder: &mut C) -> Result<Self, <C as Encoder<I>>::Error>
    where
        C: Encoder<I>,
    {
        let mut buf = BytesMut::new();
        encoder.encode(item, &mut buf)?;
        Ok(TestStream {
            input: BytesMut::new(),
            output: Cursor::new(buf),
        })
    }

    pub fn input(&self) -> &[u8] {
        &*self.input
    }
}

impl AsyncWrite for TestStream {
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        self.get_mut().input.extend_from_slice(buf);
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Poll::Ready(Ok(()))
    }
}

impl AsyncRead for TestStream {
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
    use super::TestStream;
    use actix_web::web::BytesMut;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio_util::codec::{Decoder, LinesCodec};

    #[tokio::test]
    async fn test_input() {
        let mut stream = TestStream::default();
        stream.write_all(b"Hello, world").await.unwrap();
        assert_eq!(stream.input(), b"Hello, world");
    }

    #[tokio::test]
    async fn test_output() {
        let mut codec = LinesCodec::default();
        let mut stream = TestStream::with_output("123", &mut codec).unwrap();
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await.unwrap();
        assert_eq!(
            codec.decode(&mut BytesMut::from(&*buf)).unwrap().unwrap(),
            "123"
        );
    }
}
