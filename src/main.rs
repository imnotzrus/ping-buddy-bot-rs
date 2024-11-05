use ping_buddy_lib::{setup_logger, spin_up};

#[tokio::main]
async fn main() {
  setup_logger();
  spin_up().await;
}
