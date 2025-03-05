use std::sync::Arc;

use crate::repositories::{car::CarRepositoryImpl, part::PartRepositoryImpl};

#[derive(Clone, Into)]
pub(crate) struct AppState {
    #[into]
    pub(crate) car_repository: Arc<CarRepositoryImpl>,
    #[into]
    pub(crate) part_repository: Arc<PartRepositoryImpl>,
}

#[derive(From, Clone, Deref)]
#[from(AppState)] 
#[deref(forward)]
pub(crate) struct AppStateRef(pub(crate) Arc<AppState>);

pub(crate) trait IsState: Unpin + Sized + Clone + Send + Sync + 'static {}
impl<A: Clone + Send + Sync + Unpin + Sized + 'static> IsState for A {}

use derive_more::{Deref, From, Into};
#[cfg(test)]
pub(crate) use test_state::{test_state, TestState};

#[cfg(test)]
mod test_state {
    use std::sync::Arc;

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
}
