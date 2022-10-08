pub struct Here;
pub struct There<T>(T);

pub trait Count {
    fn count() -> u32;
}

impl Count for Here {
    #[inline]
    fn count() -> u32 {
        0
    }
}

impl<N> Count for There<N>
where
    N: Count,
{
    fn count() -> u32 {
        N::count() + 1
    }
}
