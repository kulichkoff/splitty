use splitty::transport::telegram;

#[tokio::main]
async fn main() {
    telegram::serve().await;
}
