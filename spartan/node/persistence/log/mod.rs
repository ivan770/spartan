use std::{io::Error as IoError, io::SeekFrom, mem::size_of, path::Path};

use bincode::{deserialize, serialize, serialize_into, serialized_size};
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error as ThisError;
use tokio::{
    fs::OpenOptions, io::AsyncBufReadExt, io::AsyncRead, io::AsyncReadExt, io::AsyncSeek,
    io::AsyncSeekExt, io::AsyncWrite, io::AsyncWriteExt, io::BufReader, stream::StreamExt,
};

use super::PersistenceError;

type Error = PersistenceError<LogError>;

#[derive(ThisError, Debug)]
pub enum LogError {
    #[error("Unable to open file: {0}")]
    FileOpenError(IoError),
    #[error("Unable to append serialized entry to file: {0}")]
    FileAppendError(IoError),
    #[error("Unable to read entry line from file: {0}")]
    LineReadError(IoError),
}

pub struct LogConfig<'a> {
    path: &'a Path,
}

pub struct Log<'a> {
    config: LogConfig<'a>,
}

pub struct Test(u32);

impl<'a> Log<'a> {
    pub fn new(config: LogConfig<'a>) -> Self {
        Log { config }
    }

    fn make_log_entry<S>(source: &S) -> Result<Vec<u8>, Error>
    where
        S: Serialize,
    {
        let size = serialized_size(source).map_err(Error::SerializationError)?;
        let capacity = size_of::<u64>() + size as usize;

        let mut buf = Vec::with_capacity(capacity);

        buf.extend(&size.to_le_bytes());

        buf.resize(capacity, 0);

        serialize_into(&mut buf[size_of::<u64>()..], source).map_err(Error::SerializationError)?;

        Ok(buf)
    }

    async fn parse_log<T, S>(source: &mut S) -> Result<Vec<T>, Error>
    where
        S: AsyncSeek + AsyncRead + Unpin,
        T: DeserializeOwned,
    {
        let mut entries = Vec::new();

        let source_size = source
            .seek(SeekFrom::End(0))
            .await
            .map_err(|e| Error::DriverError(LogError::LineReadError(e)))?;

        source
            .seek(SeekFrom::Start(0))
            .await
            .map_err(|e| Error::DriverError(LogError::LineReadError(e)))?;

        while source
            .seek(SeekFrom::Current(0))
            .await
            .map_err(|e| Error::DriverError(LogError::LineReadError(e)))?
            < source_size
        {
            let size = source
                .read_u64_le()
                .await
                .map_err(|e| Error::DriverError(LogError::LineReadError(e)))?;

            // Might need to re-use allocations here
            let mut buf = Vec::with_capacity(size as usize);

            source
                .take(size)
                .read_buf(&mut buf)
                .await
                .map_err(|e| Error::DriverError(LogError::LineReadError(e)))?;

            entries.push(deserialize(&buf).map_err(Error::SerializationError)?);
        }

        Ok(entries)
    }

    pub async fn append<S>(&self, source: &S, destination: &Path) -> Result<(), Error>
    where
        S: Serialize,
    {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.config.path.join(destination))
            .await
            .map_err(|e| Error::DriverError(LogError::FileOpenError(e)))?
            .write_all(&Self::make_log_entry(source)?)
            .await
            .map_err(|e| Error::DriverError(LogError::FileAppendError(e)))
    }

    pub async fn load<S>(&self, source: &Path) -> Result<Vec<S>, Error>
    where
        S: DeserializeOwned,
    {
        let mut file = OpenOptions::new()
            .read(true)
            .open(self.config.path.join(source))
            .await
            .map_err(|e| Error::DriverError(LogError::FileOpenError(e)))?;

        Self::parse_log(&mut file).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    use std::io::Cursor;

    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_append_read() {
        let file = NamedTempFile::new().unwrap();

        let log = Log::new(LogConfig {
            path: Path::new("./"),
        });

        log.append(&String::from("Hello, world"), file.path())
            .await
            .unwrap();

        let entries = log.load::<String>(file.path()).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries.first().unwrap(), &String::from("Hello, world"));
    }

    #[tokio::test]
    async fn test_empty_file_load() {
        let file = NamedTempFile::new().unwrap();

        let log = Log::new(LogConfig {
            path: Path::new("./"),
        });

        let entries = log.load::<String>(file.path()).await.unwrap();
        assert!(entries.is_empty());
    }

    #[tokio::test]
    async fn test_serialize_log_entry() {
        let entry = Log::make_log_entry(&vec![1u32, 2, 3]).unwrap();
        let parsed = Log::parse_log::<Vec<u32>, _>(&mut Cursor::new(entry)).await.unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(&*parsed.first().unwrap(), &[1, 2, 3]);
    }
}
