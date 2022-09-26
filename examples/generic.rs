use coproduct::{Coproduct, Count, Embed, EmptyUnion, IndexedDrop, Union};

fn transformer<T, I, Indices>(c: Coproduct<T>) -> Coproduct<Union<u32, T::Pruned>>
where
    T: coproduct::At<I, u8> + coproduct::Without<I> + IndexedDrop,
    T::Pruned: IndexedDrop,
    Coproduct<Union<u32, T::Pruned>>: Embed<Coproduct<T::Pruned>, Indices>,
    I: Count,
{
    match c.uninject() {
        Ok(x) => Coproduct::inject(x as u32),
        Err(x) => x.embed(),
    }
}

fn main() {
    let x: Coproduct<Union<String, Union<u8, EmptyUnion>>> = Coproduct::inject(8);
    dbg!(x.clone());
    let y = transformer(x);
    dbg!(y);
}
