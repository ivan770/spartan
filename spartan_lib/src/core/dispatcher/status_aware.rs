use crate::core::{
    db::{Database, Result},
    dispatcher::simple::SimpleDispatcher,
    payload::{Dispatchable, Status},
};
use uuid::Uuid;

pub trait StatusAwareDispatcher<M>: SimpleDispatcher<M>
where
    M: Dispatchable + Status,
{
    fn pop(&mut self) -> Result<&M>;
    fn requeue(&mut self, id: &Uuid) -> Result<()>;
}

impl<T, M> StatusAwareDispatcher<M> for T
where
    T: SimpleDispatcher<M> + Database<M>,
    M: Dispatchable + Status,
{
    fn pop(&mut self) -> Result<&M> {
        let position = self.position(|msg| msg.reservable() && msg.obtainable())?;
        let message = self.get_mut(position).unwrap();
        message.reserve();
        Ok(message)
    }

    fn requeue(&mut self, id: &Uuid) -> Result<()> {
        let position =
            self.position(|msg| id == msg.id() && msg.requeueable() && msg.obtainable())?;
        self.get_mut(position).unwrap().requeue();
        Ok(())
    }
}
