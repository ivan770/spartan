use crate::node::Manager;
use actix_rt::time::delay_for;
use std::time::Duration;

/// Persistence job handler, that persists all databases from manager
pub async fn spawn_persistence(manager: &Manager<'_>) {
    debug!("Spawning persistence job.");

    if let Some(persistence) = manager.config.persistence.as_ref() {
        let timer = Duration::from_secs(persistence.config().timer);

        loop {
            delay_for(timer).await;
            if let Err(e) = manager.snapshot().await {
                error!("{}", e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use crate::{
        config::Config, node::Manager, persistence_config::Persistence,
        persistence_config::PersistenceConfig,
    };
    use spartan_lib::core::{
        dispatcher::{SimpleDispatcher, StatusAwareDispatcher},
        message::builder::MessageBuilder,
        payload::Dispatchable,
    };
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_persistence() {
        let tempdir = TempDir::new().expect("Unable to create temporary test directory");

        let config = Config {
            queues: Box::new([
                String::from("test").into_boxed_str(),
                String::from("test2").into_boxed_str(),
            ]),
            persistence: Some(Persistence::Snapshot(PersistenceConfig {
                path: Cow::Borrowed(tempdir.path()),
                timer: 10,
            })),
            ..Default::default()
        };

        {
            let manager = Manager::new(&config);

            let message = MessageBuilder::default()
                .body("Hello, world")
                .compose()
                .unwrap();

            manager
                .queue("test")
                .unwrap()
                .database()
                .await
                .push(message);

            manager.snapshot().await.unwrap();
        }

        let mut manager = Manager::new(&config);

        manager.load_from_fs().await.unwrap();

        manager.queue("test2").unwrap();
        assert_eq!(
            manager
                .queue("test")
                .unwrap()
                .database()
                .await
                .pop()
                .unwrap()
                .body(),
            "Hello, world"
        );
    }
}
