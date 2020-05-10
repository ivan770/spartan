pub trait Identifiable {
    type Id: Copy + Eq;

    fn id(&self) -> Self::Id;
}
