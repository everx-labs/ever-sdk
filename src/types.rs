use crate::*;
use std::marker::PhantomData;

pub struct Change<T> {
    pub old: T,
    pub new: T
}

pub struct NodeResponce<T> {
    phantom: PhantomData<T>
}

//impl<T> NodeResponce<T> for Future<Item = T, Error = NodeError> {
//
//}

pub struct ChangesStream<T> {
    phantom: PhantomData<T>
}