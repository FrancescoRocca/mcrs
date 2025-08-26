mod json;
mod packets;
mod server;

fn main() {
    println!("mcrs");
    let server = server::Server::new("127.0.0.1".to_string(), "25565".to_string());

    println!("Running {}:{}", server.host(), server.port());
    server.run();
}
