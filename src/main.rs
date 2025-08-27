mod json;
mod packets;
mod server;

fn main() {
    let server = server::Server::new("127.0.0.1", "25565");

    println!("Running {}:{}", server.host(), server.port());
    server.run();
}
