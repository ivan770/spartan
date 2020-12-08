use serde::{Deserialize, Serialize};

/// Message status
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Status {
    /// Message is available for reserving, if it still has any tries
    Available,

    /// Message is currently being processed, and thus is available for requeuing
    Transit,
}

impl Default for Status {
    fn default() -> Status {
        Status::Available
    }
}

/// Message state, containing try count and status
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct State {
    status: Status,
    tries: u32,
    max_tries: u32,
}

impl State {
    pub(crate) fn new(max_tries: u32) -> State {
        State {
            status: Status::default(),
            tries: 0,
            max_tries,
        }
    }

    fn has_tries(&self) -> bool {
        self.tries < self.max_tries
    }

    pub(crate) fn requeue(&mut self) {
        self.status = Status::Available;
    }

    pub(crate) fn reserve(&mut self) {
        self.status = Status::Transit;
        self.tries += 1;
    }

    pub(crate) fn requeueable(&self) -> bool {
        self.status == Status::Transit
    }

    pub(crate) fn reservable(&self) -> bool {
        self.status == Status::Available && self.has_tries()
    }

    pub(crate) fn requires_gc(&self) -> bool {
        self.tries == self.max_tries && self.status == Status::Available
    }

    /// Get current message status
    ///
    /// For more information about statuses check [`Status`]
    pub fn status(&self) -> &Status {
        &self.status
    }

    /// Get message current tries
    ///
    /// Each message reservation increases try count by 1
    pub fn tries(&self) -> &u32 {
        &self.tries
    }

    /// Get message max tries
    ///
    /// If `tries = max_tries`, then message is no longer available for
    /// reservation
    pub fn max_tries(&self) -> &u32 {
        &self.max_tries
    }
}

#[cfg(test)]
mod tests {
    use super::State;

    #[test]
    fn create_valid_state() {
        let state = State::new(1);
        assert!(state.reservable());
        assert!(!state.requeueable());
        assert!(!state.requires_gc());
    }

    #[test]
    fn create_useless_state() {
        let state = State::new(0);
        assert!(state.requires_gc());
    }

    #[test]
    fn lifecycle() {
        let mut state = State::new(1);
        assert!(state.reservable());
        state.reserve();
        assert!(state.requeueable());
        state.requeue();
        assert!(state.requires_gc());
    }
}
