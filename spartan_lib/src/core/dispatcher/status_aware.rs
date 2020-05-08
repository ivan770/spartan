use crate::core::{db::StatusAwareDatabase, dispatcher::simple::SimpleDispatcher, payload::Status};
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
    T: SimpleDispatcher<M> + StatusAwareDatabase<M, RequeueKey = Uuid>,
    M: Status,
{
    fn pop(&mut self) -> Option<&M> {
        let position = self.position(|msg| msg.reservable() && msg.obtainable())?;
        let message = self.reserve(position).unwrap();
        message.reserve();
        Some(message)
    }

    fn requeue(&mut self, key: Uuid) -> Option<()> {
        let message = self.requeue(key, |msg| msg.requeueable() && msg.obtainable())?;
        message.requeue();
        Some(())
    }
}
