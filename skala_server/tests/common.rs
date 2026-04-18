use std::fmt::Debug;

use axum_test::TestServer;
use skala_server::{
    App, Result,
    advisor::{Advice, Advisor, ReactorSnapshot},
};
use sqlx::SqlitePool;
use tokio::sync::Mutex;

pub fn setup(db_pool: SqlitePool, advisor: MockAdvisor) -> TestServer {
    let db_pool = db_pool.clone();
    let app = App::new(db_pool, u16::MAX, advisor);
    TestServer::new(app.into_router())
}

pub struct MockAdvisor {
    #[allow(clippy::type_complexity)]
    advice_fn: Mutex<Box<dyn FnMut(Vec<ReactorSnapshot>) -> Result<Advice> + Send + 'static>>,
}

impl MockAdvisor {
    pub fn new(
        advice_fn: impl FnMut(Vec<ReactorSnapshot>) -> Result<Advice> + Send + 'static,
    ) -> Self {
        let advice_fn: Box<dyn FnMut(_) -> _ + Send> = Box::new(advice_fn);
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
    async fn advise(
        &self,
        reactor_snapshots: impl IntoIterator<Item = ReactorSnapshot> + Send,
    ) -> Result<Advice> {
        self.advice_fn.lock().await(reactor_snapshots.into_iter().collect())
    }
}
