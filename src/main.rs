mod server;
mod conn;
mod cmd;
mod config;
mod log;

fn main() {
    log::init();
    cmd::parse();
}