use frunk::indices::{Here, There};

pub trait Count {
    fn count() -> u32;
}

impl Count for Here {
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
