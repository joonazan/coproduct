# Coproduct

Have you ever found yourself in a situation where you'd like to have two enums
where only a few variants differ? Usually that involves a lot of duplication
and boilerplate. Not any more! Coproducts allow you to describe them and
convert between them effortlessly!

```Rust
use coproduct::{Coproduct, MkUnion, Union};

#[derive(Debug)]
struct A;
#[derive(Debug)]
struct B;
#[derive(Debug)]
struct C;
#[derive(Debug)]
struct D;

type ABC = MkUnion!(A, B, C);

fn main() {
    let abc: Coproduct<ABC> = Coproduct::inject(A);
    let abcd: Coproduct<Union<D, ABC>> = abc.embed();
    println!("{:?}", abcd);
}
```

Find out more in the [documentation](https://docs.rs/coproduct).
