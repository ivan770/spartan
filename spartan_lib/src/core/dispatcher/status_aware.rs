use crate::core::{db::Database, dispatcher::simple::SimpleDispatcher, payload::Status};
use uuid::Uuid;

pub trait StatusAwareDispatcher<M>: SimpleDispatcher<M>
where
    M: Status,
{
    fn pop(&mut self) -> Option<&M>;
    fn requeue(&mut self, id: Uuid) -> Option<()>;
}

impl<T, M> StatusAwareDispatcher<M> for T
where
    T: SimpleDispatcher<M> + Database<M>,
    M: Status,
{
    fn pop(&mut self) -> Option<&M> {
        let position = self.position(|msg| msg.reservable() && msg.obtainable())?;
        let message = self.get_mut(position).unwrap();
        message.reserve();
        Some(message)
    }

    fn requeue(&mut self, id: Uuid) -> Option<()> {
        let position =
            self.position(|msg| id == msg.id() && msg.requeueable() && msg.obtainable())?;
        self.get_mut(position).unwrap().requeue();
        Some(())
    }
}
