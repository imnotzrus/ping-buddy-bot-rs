use ping_buddy_lib::{setup_logger, spin_up, Result};

#[tokio::main]
async fn main() -> Result {
  setup_logger();
  spin_up().await
}
