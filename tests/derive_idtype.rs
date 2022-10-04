use coproduct::{type_inequality::NotEqual, IdType};

#[derive(IdType)]
struct A;
#[derive(IdType)]
struct B;

fn require_not_equal<A, B>(_: A, _: B)
where
    A: NotEqual<B>,
{
}

#[test]
fn test() {
    require_not_equal(A, B);
}
