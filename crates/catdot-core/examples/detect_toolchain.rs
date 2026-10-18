//! Read-only toolchain probe (Part N step 1 + H1).
//! Run: `cargo run -p catdot-core --example detect_toolchain`

use catdot_core::tools::detect_toolchain;

fn main() {
    let r = detect_toolchain();
    println!("{}", serde_json::to_string_pretty(&r).expect("json"));
}
