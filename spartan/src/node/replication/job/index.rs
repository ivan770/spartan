use super::stream::Stream;

pub(super) struct RecvIndex<'a> {
    stream: &'a mut Stream,
    indexes: Vec<(Box<str>, u64)>,
}

impl<'a> RecvIndex<'a> {
    pub fn new(stream: &'a mut Stream, indexes: Vec<(Box<str>, u64)>) -> Self {
        RecvIndex { stream, indexes }
    }
}

pub(super) struct BatchAskIndex<'a> {
    batch: Vec<RecvIndex<'a>>,
}

impl<'a> BatchAskIndex<'a> {
    pub fn with_capacity(capacity: usize) -> Self {
        BatchAskIndex {
            batch: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, index: RecvIndex<'a>) {
        self.batch.push(index);
    }
}
