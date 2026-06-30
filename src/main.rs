mod ast;
mod eval;
mod repl;
mod server;

#[tokio::main]
async fn main() {
    if std::env::args().nth(1).as_deref() == Some("serve") {
        server::run().await;
    } else {
        repl::run();
    }
}
