use coproduct::{Coproduct, Count, EmptyUnion, IndexedDrop, Union};

fn transformer<T, I>(c: Coproduct<T>) -> Coproduct<Union<u32, T::Pruned>>
where
    T: coproduct::At<I, u8> + coproduct::Without<I> + IndexedDrop,
    T::Pruned: IndexedDrop,
    I: Count,
{
    match c.uninject() {
        Ok(x) => Coproduct::inject(x as u32),
        Err(x) => todo!(),
    }
}

fn main() {
    let x: Coproduct<Union<u8, EmptyUnion>> = Coproduct::inject(8);
    dbg!(x.clone());
    let y = transformer(x);
    dbg!(y);
}