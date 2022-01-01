use super::super::Transform;

pub struct Identity<T> {
    d: std::marker::PhantomData<T>
}

impl<T> Transform for Identity<T>
{
    type Value = T;

    fn transform(&self, _: &mut T) {
    }
}
