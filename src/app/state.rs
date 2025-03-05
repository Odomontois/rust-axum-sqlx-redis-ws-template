use std::{fmt::Debug, sync::Arc};

use derive_more::AsRef;

use crate::{
    models::car::Car,
    repositories::{
        car::{CarRepository, CarRepositoryImpl, HasCarRepo},
        part::{PartRepository, PartRepositoryImpl},
    },
};

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) car_repository: Arc<CarRepositoryImpl>,
    pub(crate) part_repository: Arc<PartRepositoryImpl>,
}

impl Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

pub(crate) async fn init_state(
    car_repository: Arc<CarRepositoryImpl>,
    part_repository: Arc<PartRepositoryImpl>,
) -> AppState {
    AppState {
        car_repository,
        part_repository,
    }
}

pub(crate) trait IsState: Debug + Unpin + Sized + Clone + Send + Sync + 'static {}
impl<A: Debug + Clone + Send + Sync + Unpin + Sized + 'static> IsState for A {}

impl HasCarRepo for AppState {
    type CarRepo = CarRepositoryImpl;
    fn car_repo(&self) -> Arc<Self::CarRepo> {
        self.car_repository.clone()
    }
}

pub(crate) struct TestState<A>(pub(crate) Arc<A>);

pub(crate) fn test_state<A>(state: A) -> TestState<A> {
    TestState(Arc::new(state))
}

impl<A> AsRef<Arc<A>> for TestState<A> {
    fn as_ref(&self) -> &Arc<A> {
        &self.0
    }
}

impl<A> Clone for TestState<A> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<A> Debug for TestState<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
