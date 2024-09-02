mod cmds;
mod db;
mod resp;
mod server;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // Open TCP connection at 6379
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    let db_holder = db::DbDropGuard::new();

    // Await packets
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let storage = db_holder.db();
        tokio::spawn(async move {
            server::handle_connection(stream, &storage).await;
        });
    }
}
