use snv::load_from;

fn main() {
    let _ = load_from(".env");

    let api_key = std::env::var("PATH").unwrap();
    println!("KEY: {}", api_key)
}
