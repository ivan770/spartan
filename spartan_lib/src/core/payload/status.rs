use crate::core::payload::Dispatchable;

pub trait Status: Dispatchable {
    fn requeue(&mut self);
    fn reserve(&mut self);
    fn requeueable(&self) -> bool;
    fn reservable(&self) -> bool;
}
