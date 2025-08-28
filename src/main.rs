mod json;
mod packets;
mod server;
mod utils;

fn main() {
    let mut server = server::MinecraftServer::new("127.0.0.1", "25565");

    println!("[info] Running {}:{}", server.host(), server.port());
    match server.run() {
        Ok(()) => {}
        Err(e) => {
            println!("{}", e.to_string());
        }
    }
}
