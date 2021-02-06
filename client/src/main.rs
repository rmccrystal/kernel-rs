use env_logger::Logger;

mod kdmapper;
mod cleaner;

fn main() {
    Logger::from_default_env();
    kdmapper::map_driver().unwrap();
}