use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ReplicaStorage {
    confirmed_index: u64,
}

impl Default for ReplicaStorage {
    fn default() -> Self {
        ReplicaStorage { confirmed_index: 0 }
    }
}

impl ReplicaStorage {
    pub fn confirm(&mut self) {
        self.confirmed_index += 1;
    }
}
