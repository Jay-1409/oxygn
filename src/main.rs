fn main() {
    let config = oxygen::config::load_config();
    println!("Loaded config successfully:\n{:#?}", config);
}