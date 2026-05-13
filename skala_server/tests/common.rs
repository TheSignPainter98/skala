use std::fmt::Debug;
use std::sync::{Mutex as StdMutex, OnceLock};

use axum_test::TestServer;
use log::{LevelFilter, Log, Metadata, Record};
use skala_server::{
    App, Result,
    advisor::{Advice, Advisor, PastEvent, SystemKnowledge},
};
use sqlx::SqlitePool;
use tokio::sync::Mutex;

type AdviceFn = dyn for<'event> FnMut(
        Vec<&'event PastEvent>,
        f64,
        Option<&'event SystemKnowledge>,
    ) -> Result<Advice>
    + Send
    + 'static;

pub fn setup(db_pool: SqlitePool, advisor: MockAdvisor) -> TestServer {
    let db_pool = db_pool.clone();
    let app = App::new(db_pool, u16::MAX, advisor);
    TestServer::new(app.into_router())
}

pub fn recorded_logs() -> &'static RecordedLogs {
    static LOGS: RecordedLogs = RecordedLogs::new();
    static LOGGER: TestLogger = TestLogger { logs: &LOGS };
    static INIT: OnceLock<()> = OnceLock::new();

    INIT.get_or_init(|| {
        log::set_logger(&LOGGER).expect("logger should initialise once");
        log::set_max_level(LevelFilter::Error);
    });

    &LOGS
}

pub struct RecordedLogs {
    records: StdMutex<Vec<String>>,
}

impl RecordedLogs {
    const fn new() -> Self {
        Self {
            records: StdMutex::new(Vec::new()),
        }
    }

    pub fn contains(&self, needle: &str) -> bool {
        self.records
            .lock()
            .unwrap()
            .iter()
            .any(|record| record.contains(needle))
    }
}

struct TestLogger {
    logs: &'static RecordedLogs,
}

impl Log for TestLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= log::Level::Error
    }

    fn log(&self, record: &Record<'_>) {
        if self.enabled(record.metadata()) {
            self.logs
                .records
                .lock()
                .unwrap()
                .push(record.args().to_string());
        }
    }

    fn flush(&self) {}
}

pub struct MockAdvisor {
    advice_fn: Mutex<Box<AdviceFn>>,
}

impl MockAdvisor {
    pub fn new(
        advice_fn: impl for<'event> FnMut(
            Vec<&'event PastEvent>,
            f64,
            Option<&'event SystemKnowledge>,
        ) -> Result<Advice>
        + Send
        + 'static,
    ) -> Self {
        let advice_fn: Box<AdviceFn> = Box::new(advice_fn);
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
    async fn advise<'event, I>(
        &'event self,
        past_events: I,
        target_energy_production_rate: f64,
        system_knowledge: Option<&'event SystemKnowledge>,
    ) -> Result<Advice>
    where
        I: IntoIterator<Item = &'event PastEvent> + Send,
        I::IntoIter: Send,
    {
        self.advice_fn.lock().await(
            past_events.into_iter().collect(),
            target_energy_production_rate,
            system_knowledge,
        )
    }
}
