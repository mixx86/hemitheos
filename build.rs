fn main() {
    println!("cargo::rustc-check-cfg=cfg(has_arch_impl)");

    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    if matches!(arch.as_str(), "aarch64" | "riscv64") {
        println!("cargo:rustc-cfg=has_arch_impl");
    }

    println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_ARCH");

    match arch.as_str() {
        "aarch64" => {
            println!("cargo:rerun-if-changed=src/arch/arm64/boot.s");
            println!("cargo:rerun-if-changed=src/arch/arm64/vectors.s");
            println!("cargo:rerun-if-changed=src/arch/arm64/exception_entry.s");
            println!("cargo:rerun-if-changed=src/arch/arm64/linker.ld");
            println!("cargo:rustc-link-arg=-T{manifest_dir}/src/arch/arm64/linker.ld");
            cc::Build::new()
                .file("src/arch/arm64/boot.s")
                .file("src/arch/arm64/vectors.s")
                .file("src/arch/arm64/exception_entry.s")
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
