use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;

pub trait Dispatchable: Serialize + DeserializeOwned {
    fn id(&self) -> &Uuid;
    fn obtainable(&self) -> bool;
    fn gc(&self) -> bool;
}
