fn main() {
    println!("cargo:rustc-link-search=.");
    println!("cargo:rustc-link-lib=static=urcl");
    println!("cargo:rerun-if-changed=liburcl.a")
}
