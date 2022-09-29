use coproduct::{At, Coproduct, CopyableCoproduct, Count, Embed, IndexedDrop, Union};

trait Prepend<T> {
    type With;
}

impl<T, U: IndexedDrop> Prepend<T> for Coproduct<U> {
    type With = Coproduct<Union<T, U>>;
}

impl<T: Copy, U: Copy> Prepend<T> for CopyableCoproduct<U> {
    type With = CopyableCoproduct<Union<T, U>>;
}

fn transformer<C, I, J, Indices>(c: C) -> <<C as At<I, u8>>::Pruned as Prepend<u32>>::With
where
    C: At<I, u8>,
    C::Pruned: Prepend<u32> + Embed<<<C as At<I, u8>>::Pruned as Prepend<u32>>::With, Indices>,
    <<C as At<I, u8>>::Pruned as Prepend<u32>>::With: At<J, u32>,
    I: Count,
{
    match c.uninject() {
        Ok(x) => coproduct::inject(x as u32),
        Err(x) => x.embed(),
    }
}

fn main() {
    let x: Coproduct!(String, u8) = Coproduct::inject(8);
    dbg!(x.clone());
    let y = transformer(x);
    dbg!(y);

    let x: CopyableCoproduct!(i32, u8) = CopyableCoproduct::inject(8);
    dbg!(x.clone());
    let y = transformer(x);
    dbg!(y);
}
