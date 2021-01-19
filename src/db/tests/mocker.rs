use std::marker::PhantomData;
use std::mem;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use actix::prelude::*;
use dev::MessageResponse;

pub type MockerFn<T> = Box<dyn Fn(Box<dyn Any>, &mut SyncContext<Mocker<T>>) -> Box<dyn Any>>;

/// This actor is able to wrap another actor and accept all the messages the
/// wrapped actor can, passing it to a closure which can mock the response of
/// the actor.
pub struct Mocker<T: Sized + 'static> {
    phantom: PhantomData<T>,
    mock: HashMap<TypeId, MockerFn<T>>,
    default: MockerFn<T>,
}

impl<T> Mocker<T> {
    pub fn mock(mock: MockerFn<T>) -> Mocker<T> {
        Mocker::<T> {
            phantom: PhantomData,
            mock: HashMap::new(),
            default: mock,
        }
    }

    pub fn with_handler<H: 'static>(mut self, mock: MockerFn<T>) -> Mocker<T> {
        self.mock.insert(TypeId::of::<H>(), mock);
        self
    }

    #[allow(dead_code)]
    pub fn add_handler<H: 'static>(&mut self, mock: MockerFn<T>) {
        self.mock.insert(TypeId::of::<H>(), mock);
    }
}

impl<T: Sized + 'static> Actor for Mocker<T> {
    type Context = SyncContext<Self>;
}

impl<M: 'static, T: Sized + 'static> Handler<M> for Mocker<T>
where
    M: Message,
    <M as Message>::Result: MessageResponse<Mocker<T>, M>,
{
    type Result = M::Result;
    fn handle(&mut self, msg: M, ctx: &mut Self::Context) -> M::Result {
        let mut ret = if self.mock.contains_key(&msg.type_id()) {
            (self.mock[&msg.type_id()])(Box::new(msg), ctx)
        } else {
            (self.default)(Box::new(msg), ctx)
        };
        let out = mem::replace(
            ret.downcast_mut::<Option<M::Result>>()
                .expect("wrong return type for message"),
            None,
        );
        match out {
            Some(a) => a,
            _ => panic!(),
        }
    }
}
