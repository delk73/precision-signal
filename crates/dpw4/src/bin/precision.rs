#![forbid(unsafe_code)]

#[path = "common/mod.rs"]
mod common;
#[path = "precision/mod.rs"]
mod precision_app;

fn main() {
    common::exit_with_result(precision_app::run());
}
