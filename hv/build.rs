fn main() {
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    match arch.as_str() {
        "aarch64" => {
            println!("cargo:rerun-if-changed=asm/aarch64/boot.s");
            println!("cargo:rerun-if-changed=asm/aarch64/vectors.s");
            println!("cargo:rerun-if-changed=asm/aarch64/linker.ld");
            println!("cargo:rustc-link-arg=-T{manifest_dir}/asm/aarch64/linker.ld");
            cc::Build::new()
                .file("asm/aarch64/boot.s")
                .file("asm/aarch64/vectors.s")
                .compile("startup");
        }
        // riscv64: reserved for future support, not implemented yet
        "riscv64" => {
            println!("cargo:rerun-if-changed=asm/riscv64/boot.s");
            println!("cargo:rerun-if-changed=asm/riscv64/linker.ld");
            println!("cargo:rustc-link-arg=-T{manifest_dir}/asm/riscv64/linker.ld");
            cc::Build::new()
                .file("asm/riscv64/boot.s")
                .compile("startup");
        }
        _ => (),
    }
}
