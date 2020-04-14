use crate::core::{
    db::{Database, Result},
    payload::Dispatchable,
};
use uuid::Uuid;

pub trait SimpleDispatcher<M>
where
    M: Dispatchable,
{
    fn push(&mut self, message: M) -> Result<()>;
    fn peak(&self) -> Result<&M>;
    fn delete(&mut self, id: &Uuid) -> Result<()>;
    fn gc(&mut self) -> Result<()>;
    fn size(&self) -> Result<usize>;
    fn clear(&mut self) -> Result<()>;
}

impl<T, M> SimpleDispatcher<M> for T
where
    T: Database<M>,
    M: Dispatchable,
{
    fn push(&mut self, message: M) -> Result<()> {
        self.push_raw(message)
    }

    fn peak(&self) -> Result<&M> {
        Ok(self.get(self.position(|msg| msg.obtainable())?).unwrap())
    }

    fn gc(&mut self) -> Result<()> {
        self.retain(|msg| !msg.gc())
    }

    fn delete(&mut self, id: &Uuid) -> Result<()> {
        Ok(self
            .delete_pos(self.position(|msg| msg.id() == id)?)
            .unwrap())
    }

    fn size(&self) -> Result<usize> {
        self.len()
    }

    fn clear(&mut self) -> Result<()> {
        self.clear()
    }
}
