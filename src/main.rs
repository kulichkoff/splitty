use std::{env, sync::Arc};

use splitty::{AppContext, db, postgres::SqlxPartyRepository, transport::telegram};

#[tokio::main]
async fn main() {
    let database_url =
        env::var("DATABASE_URL").expect("no DATABASE_URL environment variable presented");
    let pool = db::create_pool(&database_url)
        .await
        .expect("failed to create db pool");

    let party_repo = SqlxPartyRepository::new(pool);

    let ctx = AppContext {
        party_repo: Arc::new(party_repo),
    };

    telegram::serve(ctx).await;
}
