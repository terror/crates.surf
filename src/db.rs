use super::*;

#[derive(Debug)]
pub(crate) struct Db {
  _pool: PgPool,
}

impl Db {
  pub(crate) async fn connect(db_name: &str) -> Result<Self> {
    let db_url =
      format!("postgresql://postgres:postgres@localhost:5432/{}", db_name);

    if !Postgres::database_exists(&db_url).await? {
      Postgres::create_database(&db_url).await?;
    }

    let options = sqlx::postgres::PgConnectOptions::from_str(&db_url)?;

    let pool = PgPool::connect_with(options).await?;

    info!("Connected to PostgreSQL");

    // sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(Self { _pool: pool })
  }
}
