use crate::core::{db::StatusAwareDatabase, dispatcher::simple::SimpleDispatcher, payload::{Identifiable, Status}};

pub trait StatusAwareDispatcher<M>: SimpleDispatcher<M>
where
    M: Status,
{
    fn pop(&mut self) -> Option<&M>;
    fn requeue(&mut self, id: <M as Identifiable>::Id) -> Option<()>;
}

impl<T, M> StatusAwareDispatcher<M> for T
where
    T: SimpleDispatcher<M> + StatusAwareDatabase<M, RequeueKey = <M as Identifiable>::Id>,
    M: Status,
{
    fn pop(&mut self) -> Option<&M> {
        let position = self.position(|msg| msg.reservable() && msg.obtainable())?;
        let message = self.reserve(position).unwrap();
        message.reserve();
        Some(message)
    }

    fn requeue(&mut self, key: <M as Identifiable>::Id) -> Option<()> {
        let message = self.requeue(key, |msg| msg.requeueable() && msg.obtainable())?;
        message.requeue();
        Some(())
    }
}
