pub fn main() {
    if std::env::var("__BUILD_WITH_SIGN") != Ok("yes".to_string()) {
        panic!("do not `cargo build` directly, use `x.py` instead");
    }
    println!("cargo::rustc-link-lib=framework=Hypervisor");
    println!("link-arg=-mmacosx-version-min=11.0");
}
