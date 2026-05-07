#[derive(Debug, Hash,Eq)]
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

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index &&
        self.generation == other.generation
    }
}

