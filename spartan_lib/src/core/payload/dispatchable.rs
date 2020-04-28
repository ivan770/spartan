use super::Identifiable;

pub trait Dispatchable: Identifiable {
    fn obtainable(&self) -> bool;
    fn gc(&self) -> bool;
}
