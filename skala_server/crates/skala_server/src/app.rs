use std::ops::Deref;
use std::sync::Arc;

use axum::Router;
use sqlx::SqlitePool;

use crate::{advisor::Advisor, routes};

pub struct App {
    router: Router<()>,
}

impl App {
    pub fn new<A>(db_pool: SqlitePool, reactor_snapshot_window_limit: u16, advisor: A) -> Self
    where
        A: Advisor + 'static,
    {
        let app_state = AppState::new(db_pool, reactor_snapshot_window_limit, advisor);
        let router = routes::register(Router::new()).with_state(app_state);
        Self { router }
    }

    pub fn into_router(self) -> Router<()> {
        let Self { router } = self;
        router
    }
}
#[derive(Debug)]
pub(crate) struct AppState<A>(Arc<AppStateInner<A>>);

impl<A> Clone for AppState<A> {
    fn clone(&self) -> Self {
        let Self(inner) = self;
        Self(inner.clone())
    }
}

impl<A: Advisor> AppState<A> {
    fn new(db_pool: SqlitePool, reactor_snapshot_window_limit: u16, advisor: A) -> Self {
        Self(Arc::new(AppStateInner {
            db_pool,
            reactor_snapshot_window_limit,
            advisor,
        }))
    }
}

impl<A> Deref for AppState<A> {
    type Target = AppStateInner<A>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub(crate) struct AppStateInner<A> {
    pub(crate) db_pool: SqlitePool,
    pub(crate) reactor_snapshot_window_limit: u16,
    #[allow(unused)]
    pub(crate) advisor: A,
}
