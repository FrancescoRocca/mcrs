mod json;
mod packets;
mod responses;
mod server;
mod utils;

fn main() {
    let mut server = server::MinecraftServer::new("0.0.0.0", "25565");

    println!("[info] Running {}:{}", server.host(), server.port());
    match server.run() {
        Ok(()) => {}
        Err(e) => {
            println!("{}", e);
        }
    }
}
