//! Dirty methods and structures that we don't want to expose to the rest of the code
//! but necessary for adapting hyper crate
extern crate tower_service;

use futures::Future;

use hyper::{Body, Request, Response};
use std::error::Error as StdError;
use std::task;
use std::task::Poll;
use std::marker::PhantomData;

// Functions copied and modified from hyper::service::util.
pub fn make_payload_service<F, Target, Ret, T>(f: F, payload: T) -> MakePayloadServiceFn<F, T>
where
    F: FnMut(&Target, T) -> Ret,
    Ret: Future,
    T: Clone,
{
    MakePayloadServiceFn { f, payload }
}

pub struct MakePayloadServiceFn<F, T> {
    f: F,
    payload: T,
}

impl<'t, F, Ret, Target, Svc, MkErr, T> tower_service::Service<&'t Target> for MakePayloadServiceFn<F, T>
where
    F: FnMut(&Target, T) -> Ret,
    Ret: Future<Output = Result<Svc, MkErr>>,
    MkErr: Into<Box<dyn StdError + Send + Sync>>,
    T: Clone,
{
    type Error = MkErr;
    type Response = Svc;
    type Future = Ret;

    fn poll_ready(&mut self, _cx: &mut task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, target: &'t Target) -> Self::Future {
        (self.f)(target, self.payload.clone())
    }
}

pub fn payload_service<F, R, S, T>(f: F, payload: T) -> PayloadServiceFn<F, R, T>
where
    F: FnMut(Request<R>, T) -> S,
    S: Future,
    T: Clone,
{
    PayloadServiceFn {
        f,
        payload,
        _req: PhantomData,
    }
}

// Not exported from crate as this will likely be replaced with `impl Service`.
pub struct PayloadServiceFn<F, R, T> {
    f: F,
    payload: T,
    _req: PhantomData<fn(R)>,
}

impl<F, Ret, E, T> tower_service::Service<hyper::Request<Body>>
    for PayloadServiceFn<F, Body, T>
where
    F: FnMut(Request<Body>, T) -> Ret,
    Ret: Future<Output = Result<Response<Body>, E>>,
    E: Into<Box<dyn StdError + Send + Sync>>,
    T: Clone,
{
    type Response = hyper::Response<Body>;
    type Error = E;
    type Future = Ret;

    fn poll_ready(&mut self, _cx: &mut task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        (self.f)(req, self.payload.clone())
    }
}


