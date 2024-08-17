mod config;

fn main() {
    println!("Hello, world!");
    let _configs = match config::parse_configs() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            panic!();
        }
    };
}
