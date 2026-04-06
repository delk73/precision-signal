#![forbid(unsafe_code)]

#[path = "precision/mod.rs"]
mod precision_app;

fn main() {
    precision_app::main();
}
