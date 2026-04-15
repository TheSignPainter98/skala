use std::fmt::Debug;

use axum_test::TestServer;
use skala_server::{
    App, IntactReactorState, Result,
    advisor::{Advice, Advisor},
};
use sqlx::SqlitePool;
use tokio::sync::Mutex;

pub fn setup(db_pool: SqlitePool, advisor: MockAdvisor) -> TestServer {
    let db_pool = db_pool.clone();
    let app = App::new(db_pool, advisor);
    TestServer::new(app.into_router())
}

pub struct MockAdvisor {
    advice_fn: Mutex<Box<dyn FnMut(IntactReactorState) -> Result<Advice> + Send + 'static>>,
}

impl MockAdvisor {
    pub fn new(
        advice_fn: impl FnMut(IntactReactorState) -> Result<Advice> + Send + 'static,
    ) -> Self {
        let advice_fn: Box<dyn FnMut(IntactReactorState) -> Result<Advice> + Send> =
            Box::new(advice_fn);
        let advice_fn = Mutex::new(advice_fn);
        Self { advice_fn }
    }
}

impl Debug for MockAdvisor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MockAdvisor").finish_non_exhaustive()
    }
}

impl Advisor for MockAdvisor {
    async fn advise(&self, reactor_state: IntactReactorState) -> Result<Advice> {
        self.advice_fn.lock().await(reactor_state)
    }
}
