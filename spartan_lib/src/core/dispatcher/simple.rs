use crate::core::{db::Database, payload::Dispatchable};
use uuid::Uuid;

pub trait SimpleDispatcher<M>
where
    M: Dispatchable,
{
    fn push(&mut self, message: M);
    fn peak(&self) -> Option<&M>;
    fn delete(&mut self, id: Uuid) -> Option<()>;
    fn gc(&mut self);
    fn size(&self) -> usize;
    fn clear(&mut self);
}

impl<T, M> SimpleDispatcher<M> for T
where
    T: Database<M>,
    M: Dispatchable,
{
    fn push(&mut self, message: M) {
        self.push_raw(message);
    }

    fn peak(&self) -> Option<&M> {
        self.get(self.position(|msg| msg.obtainable())?)
    }

    fn gc(&mut self) {
        self.retain(|msg| !msg.gc());
    }

    fn delete(&mut self, id: Uuid) -> Option<()> {
        self.delete_pos(self.position(|msg| msg.id() == id)?)
    }

    fn size(&self) -> usize {
        self.len()
    }

    fn clear(&mut self) {
        self.clear()
    }
}
