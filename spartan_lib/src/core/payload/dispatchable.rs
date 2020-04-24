use super::Identifiable;
use serde::{de::DeserializeOwned, Serialize};

pub trait Dispatchable: Identifiable + Serialize + DeserializeOwned {
    fn obtainable(&self) -> bool;
    fn gc(&self) -> bool;
}
