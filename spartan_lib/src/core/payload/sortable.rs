pub trait Sortable {
    type Sort: Ord;

    fn sort(&self) -> Self::Sort;
}
