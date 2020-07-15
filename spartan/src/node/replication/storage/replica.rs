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
    pub fn get_index(&self) -> u64 {
        self.confirmed_index
    }

    pub fn confirm(&mut self, index: u64) {
        self.confirmed_index = index;
    }
}
