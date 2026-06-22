## snv - Simple .env Loader

This loads your .env files when developing. It's simple to use. Not much to it!


### Installation
```bash
cargo add snv
```

### Usage
```rust
use snv::load;

fn main() {
    let _ = load();
    let api_key = std::env::var("API_KEY").unwrap();
    println!("KEY: {}", api_key)
}

```

Alternatively, you can specify the relative path

```rust
use snv::load_from;

fn main() {
    let _ = load_from(".env.sample");

    let api_key = std::env::var("API_KEY").unwrap();
    println!("KEY: {}", api_key)
}

```

### Documentation
- `load()` loads `.env` from the current working directory
- `load_from()` takes relative or absolute paths
- Double quotes are unescaped for `\n`, `\t`, `\r`, `\'`, `\\`
- Single quotes are stripped but unescaped