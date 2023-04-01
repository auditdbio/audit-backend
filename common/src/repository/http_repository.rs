use std::marker::PhantomData;

use super::Repository;

pub trait HasDomain {
    fn get_domain() -> String;
}

pub struct HttpRepository<T> {
    _ph: PhantomData<T>,
}
