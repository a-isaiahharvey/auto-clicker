pub mod gui;
pub mod window;

#[tokio::main]
async fn main() {
    window::run().await;
}
