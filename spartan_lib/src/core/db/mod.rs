pub mod tree;
pub mod vec;

pub trait Database<M>: Default {
    type PositionKey;

    fn push_raw(&mut self, message: M);
    fn position<F>(&self, predicate: F) -> Option<Self::PositionKey>
    where
        F: Fn(&M) -> bool;
    fn get(&self, position: Self::PositionKey) -> Option<&M>;
    fn get_mut(&mut self, position: Self::PositionKey) -> Option<&mut M>;
    fn delete_pos(&mut self, position: Self::PositionKey) -> Option<()>;
    fn retain<F>(&mut self, predicate: F)
    where
        F: Fn(&M) -> bool;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn clear(&mut self);
}
