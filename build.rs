fn main() {
    // The nix-provided gcc/mold toolchain does not search the system multiarch
    // library directories by default, so the libpulse link names emitted by
    // libpulse-sys cannot be resolved. Add them explicitly.
    for dir in ["/usr/lib/x86_64-linux-gnu", "/lib/x86_64-linux-gnu"] {
        if std::path::Path::new(dir).is_dir() {
            println!("cargo:rustc-link-search=native={dir}");
        }
    }
}
