fn main() {
    let config = types::config::load_config();
    println!("Loaded config successfully:\n{:#?}", config);
}
