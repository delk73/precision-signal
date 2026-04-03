#![forbid(unsafe_code)]

#[path = "precision/mod.rs"]
mod precision_app;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    precision_app::main()
}
