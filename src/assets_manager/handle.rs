#[derive( Debug, PartialEq, Eq, Hash)]
pub struct Handle<T> {
    pub index: u32,
    pub generation: u32,
    pub _marker: std::marker::PhantomData<T>,
}

impl<T> Copy for Handle<T> {}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}