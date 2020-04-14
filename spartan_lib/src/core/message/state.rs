use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
enum Status {
    Available,
    Transit,
}

impl Default for Status {
    fn default() -> Status {
        Status::Available
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct State {
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

    fn check_tries(&self) -> bool {
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
        self.status == Status::Available && self.check_tries()
    }

    pub(crate) fn gc(&self) -> bool {
        self.tries == self.max_tries && self.status == Status::Available
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
        assert!(!state.gc());
    }

    #[test]
    fn create_useless_state() {
        let state = State::new(0);
        assert!(state.gc());
    }

    #[test]
    fn lifecycle() {
        let mut state = State::new(1);
        assert!(state.reservable());
        state.reserve();
        assert!(state.requeueable());
        state.requeue();
        assert!(state.gc());
    }
}
