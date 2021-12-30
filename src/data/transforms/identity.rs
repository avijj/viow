use super::super::Transform;

pub struct Identity<T> {
    d: std::marker::PhantomData<T>
}

impl<T> Transform for Identity<T>
{
    type InValue = T;
    type OutValue = T;

    fn transform(&self, value: T) -> T {
        value
    }
}
