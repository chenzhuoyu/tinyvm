#[cfg(target_arch = "aarch64")]
const VIRTOS_FILE: &str = "src/aarch64/virtos/asm.s";

pub fn main() {
    if std::env::var("__BUILD_WITH_SIGN") != Ok("yes".to_string()) {
        panic!("do not `cargo build` directly, use `x.py` instead");
    }
    cc::Build::new().file(VIRTOS_FILE).compile("os");
    println!("cargo::rerun-if-changed={VIRTOS_FILE}");
    println!("cargo::rustc-link-lib=os");
    println!("cargo::rustc-link-lib=framework=Hypervisor");
    println!("link-arg=-mmacosx-version-min=11.0");
}
