use std::marker::PhantomData;

pub struct Here;
pub struct There<T>(PhantomData<T>);

pub trait Counter {
    fn count() -> u32;
}

impl Counter for Here {
    fn count() -> u32 {
        0
    }
}

impl<N> Counter for There<N>
where
    N: Counter,
{
    fn count() -> u32 {
        N::count() + 1
    }
}
