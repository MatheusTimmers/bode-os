use std::env;

fn main() {
    println!("cargo::rerun-if-changed=kernel.ld");
    println!("cargo::rerun-if-changed=boards");

    if env::var("CARGO_CFG_TARGET_ARCH").as_deref() != Ok("riscv32") {
        return;
    }

    let board = if env::var_os("CARGO_FEATURE_IBEX").is_some() {
        "ibex"
    } else {
        "virt"
    };

    let root = env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo::rustc-link-arg-bins=-L{root}/boards/{board}");
    println!("cargo::rustc-link-arg-bins=-T{root}/kernel.ld");
}
