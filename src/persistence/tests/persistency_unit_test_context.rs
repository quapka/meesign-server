use tokio::sync::Mutex;

use diesel::{Connection, PgConnection};
use diesel_async::{
    pooled_connection::AsyncDieselConnectionManager, AsyncConnection, AsyncPgConnection,
};
use diesel_migrations::MigrationHarness;

use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::core::IntoContainerPort;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use testcontainers_modules::testcontainers::ImageExt;
use testcontainers_modules::testcontainers::{ContainerAsync, ReuseDirective};

use diesel_async::pooled_connection::bb8::Pool;

use crate::persistence::{error::PersistenceError, repository::MIGRATIONS};
use diesel_async::pooled_connection::bb8::PooledConnection;
use rand::distributions::Alphanumeric;
use rand::Rng;

pub fn initialize_db(database_url: &str) {
    let mut connection = PgConnection::establish(database_url).expect(&format!(
        "Couldn't connect to the test DB using connection URL: {database_url}"
    ));
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Couldn't run migrations");
}

pub struct PersistencyUnitTestContext {
    _container: ContainerAsync<Postgres>,
    pub database_url: String,
    pub pool: Pool<AsyncPgConnection>,
}

use std::sync::OnceLock;
use maybe_once::tokio::{Data, MaybeOnceAsync};

type MaybeOnceType = (Pool<AsyncDieselConnectionManager<AsyncPgConnection>>, ContainerAsync<Postgres>);

/// Initializer function.
/// Starts a Postgres container shared between all tests.
/// It will be stopped when the tests terminate.
async fn init() -> MaybeOnceType {
    // startup the container
    let password = "postgres";
    let db_name = "postgres";
    let user = "postgres";
    let default_postgres_port = 5432;

    let node = Postgres::default()
        .with_host_auth()
        .with_db_name(db_name)
        .with_user(user)
        .with_password(&password)
        .start()
        .await
        .expect(
            // FIXME TEST_DATABASE_URL is not implemented atm.
            "Could not start test Postgres database Docker container. \
            Is Docker installed? Also, try setting `TEST_DATABASE_URL` to test against \
            already started container.",
        );

    let host = node.get_host().await.unwrap();
    let host_port = node
        .get_host_port_ipv4(default_postgres_port)
        .await
        .unwrap();
    let database_url = format!("postgres://{user}:{password}@{host}:{host_port}/{db_name}");
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&database_url);
    let pool = Pool::builder()
        .max_size(20)
        .build(manager)
        .await
        .expect("Failed to create test pool");

    (pool, node)
}

/// A function that holds a static reference to the container
pub async fn data(serial: bool) -> Data<'static, MaybeOnceType> {
    static DATA: OnceLock<MaybeOnceAsync<MaybeOnceType>> = OnceLock::new();
    DATA.get_or_init(|| MaybeOnceAsync::new(|| Box::pin(init())))
        .data(serial)
        .await
}

impl PersistencyUnitTestContext {
    pub async fn new() -> Self {
        // check if TEST_DATABASE_URL exists and use that one
        let mut rng = rand::thread_rng();

        let password: String = (0..20).map(|_| rng.sample(Alphanumeric) as char).collect();
        let db_name = "meesign";
        let user = "meesign";
        let default_postgres_port = 5432;

        let container = Postgres::default()
            .with_host_auth()
            .with_db_name(db_name)
            .with_user(user)
            .with_password(&password)
            .start()
            .await
            .expect(
                "Could not start test Postgres database Docker container. \
                Is Docker installed? Also, try setting `TEST_DATABASE_URL` to test against \
                already started container.",
            );

        // Unwrapping on both host and host_port as there is not much we can do to recover.
        let host = container.get_host().await.unwrap();
        let host_port = container
            .get_host_port_ipv4(default_postgres_port)
            .await
            .unwrap();

        let database_url = format!("postgres://{user}:{password}@{host}:{host_port}/{db_name}");
        initialize_db(&database_url);

        let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&database_url);
        let pool = Pool::builder()
            .max_size(20)
            .build(manager)
            .await
            .expect("Failed to create test pool");

        Self {
            _container: container,
            database_url,
            pool,
        }
    }

    pub fn pool(&self) -> &Pool<AsyncPgConnection> {
        &self.pool
    }

    pub async fn get_test_connection(&self) -> Result<AsyncPgConnection, PersistenceError> {
        // let mut connection = self.pool().get().await.expect("Could not connect to test database through pool");
        let mut connection = AsyncPgConnection::establish(&self.database_url).await?;
        connection.begin_test_transaction().await?;
        Ok(connection)
    }
}
