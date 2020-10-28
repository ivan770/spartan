use std::{io::SeekFrom, mem::size_of, path::Path};

use bincode::{deserialize, serialize_into, serialized_size};
use cfg_if::cfg_if;
use once_cell::sync::OnceCell;
use serde::{de::DeserializeOwned, Serialize};
use tokio::{fs::OpenOptions, io::AsyncRead, io::AsyncReadExt, io::AsyncSeek, io::AsyncSeekExt, io::AsyncWriteExt, fs::remove_file};

use crate::{
    config::persistence::PersistenceConfig,
    node::{
        event::{Event, EventLog},
        Queue,
        persistence::snapshot::Snapshot
    },
};

use super::PersistenceError;

#[cfg(feature = "replication")]
use crate::node::persistence::snapshot::{REPLICATION_FILE as SNAPSHOT_REPLICATION_FILE};

const QUEUE_FILE: &str = "queue_log";

const QUEUE_COMPACTION_FILE: &str = "queue_compacted_log";

pub struct Log<'a> {
    config: &'a PersistenceConfig<'a>,
    snapshot: OnceCell<Snapshot<'a>>
}

impl<'a> Log<'a> {
    pub fn new(config: &'a PersistenceConfig) -> Self {
        Log {
            config,
            snapshot: OnceCell::new()
        }
    }

    fn make_log_entry<S>(source: &S) -> Result<Vec<u8>, PersistenceError>
    where
        S: Serialize,
    {
        let size = serialized_size(source).map_err(PersistenceError::SerializationError)?;
        let capacity = size_of::<u64>() + size as usize;

        let mut buf = Vec::with_capacity(capacity);

        buf.extend(&size.to_le_bytes());

        buf.resize(capacity, 0);

        serialize_into(&mut buf[size_of::<u64>()..], source)
            .map_err(PersistenceError::SerializationError)?;

        Ok(buf)
    }

    async fn parse_log<T, S>(source: &mut S) -> Result<Vec<T>, PersistenceError>
    where
        S: AsyncSeek + AsyncRead + Unpin,
        T: DeserializeOwned,
    {
        let mut entries = Vec::new();

        let source_size = source
            .seek(SeekFrom::End(0))
            .await
            .map_err(PersistenceError::LineReadError)?;

        source
            .seek(SeekFrom::Start(0))
            .await
            .map_err(PersistenceError::LineReadError)?;

        while source
            .seek(SeekFrom::Current(0))
            .await
            .map_err(PersistenceError::LineReadError)?
            < source_size
        {
            let size = source
                .read_u64_le()
                .await
                .map_err(PersistenceError::LineReadError)?;

            // Might need to re-use allocations here
            let mut buf = Vec::with_capacity(size as usize);

            source
                .take(size)
                .read_buf(&mut buf)
                .await
                .map_err(PersistenceError::LineReadError)?;

            entries.push(deserialize(&buf).map_err(PersistenceError::SerializationError)?);
        }

        Ok(entries)
    }

    pub async fn append<P, S>(&self, source: &S, destination: P) -> Result<(), PersistenceError>
    where
        P: AsRef<Path>,
        S: Serialize,
    {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.config.path.join(destination))
            .await
            .map_err(PersistenceError::FileReadError)?
            .write_all(&Self::make_log_entry(source)?)
            .await
            .map_err(PersistenceError::FileWriteError)
    }

    pub async fn load<S, P>(&self, source: P) -> Result<Vec<S>, PersistenceError>
    where
        S: DeserializeOwned,
        P: AsRef<Path>,
    {
        let mut file = OpenOptions::new()
            .read(true)
            .open(self.config.path.join(source))
            .await
            .map_err(PersistenceError::FileReadError)?;

        Self::parse_log(&mut file).await
    }

    pub async fn persist_event<P>(
        &self,
        event: &Event<'_>,
        source: P,
    ) -> Result<(), PersistenceError>
    where
        P: AsRef<Path>,
    {
        self.append(event, source.as_ref().join(QUEUE_FILE)).await
    }

    pub async fn load_queue<P, DB>(&self, source: P) -> Result<Queue<DB>, PersistenceError>
    where
        P: AsRef<Path>,
        DB: EventLog<Vec<Event<'static>>> + Serialize + DeserializeOwned,
    {
        let events = self
            .load::<Event, _>(source.as_ref().join(QUEUE_FILE))
            .await?;

        let database = if self.config.compaction {
            let compaction_path = source.as_ref().join(QUEUE_COMPACTION_FILE);

            match self.get_snapshot().load::<DB, _>(&compaction_path).await {
                Ok(mut database) => {
                    database.apply_log(events);

                    self.get_snapshot()
                        .persist(&database, &compaction_path)
                        .await?;

                    self.prune(&source).await?;

                    database
                },
                Err(PersistenceError::FileReadError(e)) => {
                    error!("Compaction file read error: {}", e);
                    DB::from_log(events)
                },
                Err(e) => return Err(e)
            }
        } else {
            DB::from_log(events)
        };

        cfg_if! {
            if #[cfg(feature = "replication")] {
                // Thanks to GC threshold, it's currently impossible to use log driver
                let replication_storage = self.get_snapshot().load(source.as_ref().join(SNAPSHOT_REPLICATION_FILE)).await?;
                let queue = Queue::new(database, replication_storage);
            } else {
                let queue = Queue::new(database);
            }
        }

        Ok(queue)
    }

    async fn prune<P>(&self, queue: P) -> Result<(), PersistenceError>
    where
        P: AsRef<Path>
    {
        remove_file(queue.as_ref().join(QUEUE_FILE))
            .await
            .map_err(PersistenceError::GenericIoError)
    }

    fn get_snapshot(&self) -> &Snapshot<'_> {
        self.snapshot.get_or_init(|| Snapshot::new(self.config))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{borrow::Cow, io::Cursor};

    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_append_read() {
        let file = NamedTempFile::new().unwrap();
        let config = PersistenceConfig {
            path: Cow::Borrowed(file.path().parent().unwrap()),
            ..Default::default()
        };

        let log = Log::new(&config);

        log.append(&String::from("Hello, world"), file.path())
            .await
            .unwrap();

        let entries = log.load::<String, _>(file.path()).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries.first().unwrap(), &String::from("Hello, world"));
    }

    #[tokio::test]
    async fn test_empty_file_load() {
        let file = NamedTempFile::new().unwrap();
        let config = PersistenceConfig {
            path: Cow::Borrowed(file.path().parent().unwrap()),
            ..Default::default()
        };

        let log = Log::new(&config);

        let entries = log.load::<String, _>(file.path()).await.unwrap();
        assert!(entries.is_empty());
    }

    #[tokio::test]
    async fn test_serialize_log_entry() {
        let entry = Log::make_log_entry(&vec![1u32, 2, 3]).unwrap();
        let parsed = Log::parse_log::<Vec<u32>, _>(&mut Cursor::new(entry))
            .await
            .unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(&*parsed.first().unwrap(), &[1, 2, 3]);
    }

    #[tokio::test]
    async fn test_multiple_log_entries() {
        let mut entries = Vec::new();
        entries.append(&mut Log::make_log_entry(&vec![1u32, 2, 3]).unwrap());
        entries.append(&mut Log::make_log_entry(&vec![4, 5, 6]).unwrap());
        entries.append(&mut Log::make_log_entry(&vec![7, 8, 9]).unwrap());
        let parsed = Log::parse_log::<Vec<u32>, _>(&mut Cursor::new(entries))
            .await
            .unwrap();
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed, vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]]);
    }
}
