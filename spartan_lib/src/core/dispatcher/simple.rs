use crate::core::{
    db::Database,
    payload::{Dispatchable, Identifiable},
};

pub trait SimpleDispatcher<M>
where
    M: Dispatchable,
{
    fn push(&mut self, message: M);
    fn peak(&self) -> Option<&M>;
    fn gc(&mut self);
    fn size(&self) -> usize;
    fn clear(&mut self);
}

pub trait Delete<M>: SimpleDispatcher<M>
where
    M: Dispatchable,
{
    fn delete(&mut self, id: <M as Identifiable>::Id) -> Option<()>;
}

pub trait PositionBasedDelete<M>: SimpleDispatcher<M>
where
    M: Dispatchable,
{
    fn delete(&mut self, id: <M as Identifiable>::Id) -> Option<()>;
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

    fn size(&self) -> usize {
        self.len()
    }

    fn clear(&mut self) {
        self.clear()
    }
}

impl<T, M> Delete<M> for T
where
    T: Database<M>,
    M: Dispatchable,
{
    fn delete(&mut self, id: <M as Identifiable>::Id) -> Option<()> {
        self.delete_pos(self.position(|msg| msg.id() == id)?)
    }
}

impl<T, M> PositionBasedDelete<M> for T
where
    T: Database<M, PositionKey = <M as Identifiable>::Id>,
    M: Dispatchable,
{
    fn delete(&mut self, id: <M as Identifiable>::Id) -> Option<()> {
        self.delete_pos(id)
    }
}
