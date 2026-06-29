fn main() {
    // Expose target triple as compile-time env variable TARGET
    if let Ok(target) = std::env::var("TARGET") {
        println!("cargo:rustc-env=TARGET={}", target);
    } else {
        // Fallback if not run by cargo or TARGET is unset
        println!("cargo:rustc-env=TARGET=unknown");
    }
}
