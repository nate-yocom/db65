fn main() {
    println!("cargo:rerun-if-changed=sim65/6502.c");
    cc::Build::new()
        .file("sim65/6502.c")
        .define("DB65", "1")
        .compile("sim65");
}
