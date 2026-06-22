use snv;

fn main() {
    let _ = snv::load_from(".env.sample");

    let api_key = std::env::var("API_KEY").unwrap();
    println!("KEY: {}", api_key)
}
