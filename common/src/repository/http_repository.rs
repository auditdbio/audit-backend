use std::marker::PhantomData;

pub trait HasDomain {
    fn get_domain() -> String;
}

pub struct HttpRepository<T> {
    _ph: PhantomData<T>,
}
