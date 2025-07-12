use std::{marker::PhantomData, task::Poll};

pub struct ActorTowerSerivce<Req, Res> {
    phantom: PhantomData<(Req, Res)>,
}
pub enum ActorTowerError {}
impl<Req, Res> tower::Service<Req> for ActorTowerSerivce<Req, Res> {
    type Error = ActorTowerError;
    type Future = impl Future<Output = Result<Res, Self::Error>>;
    type Response = Res;
    fn call(&mut self, req: Req) -> <ActorTowerSerivce<Req, Res> as tower::Service<Req>>::Future {
        async { todo!() }
    }
    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        todo!()
    }
}
