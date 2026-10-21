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
            println!("cargo:rerun-if-changed=src/include/linkage.h");
            println!("cargo:rerun-if-changed=src/arch/arm/include/asm/sysregs.h");
            println!("cargo:rerun-if-changed=src/arch/arm/head.S");
            println!("cargo:rerun-if-changed=src/arch/arm/entry.S");
            println!("cargo:rerun-if-changed=src/arch/arm/mmu/head.S");
            println!("cargo:rerun-if-changed=src/arch/arm/hemitheos.lds.S");
            let out_dir = std::env::var("OUT_DIR").unwrap();
            let lds_out = format!("{out_dir}/hemitheos.lds");
            std::process::Command::new("aarch64-linux-gnu-gcc")
                .args([
                    "-E",
                    "-P",
                    "-x",
                    "c",
                    "-I",
                    &format!("{manifest_dir}/src/include"),
                    "-I",
                    &format!("{manifest_dir}/src/arch/arm/include"),
                    &format!("{manifest_dir}/src/arch/arm/hemitheos.lds.S"),
                    "-o",
                    &lds_out,
                ])
                .status()
                .expect("failed to preprocess linker script");
            println!("cargo:rustc-link-arg=-T{lds_out}");
            cc::Build::new()
                .file("src/arch/arm/head.S")
                .file("src/arch/arm/entry.S")
                .file("src/arch/arm/mmu/head.S")
                .include("src/include")
                .include("src/arch/arm/include")
                .compile("startup");
        }
        // riscv64: reserved for future support, not implemented yet
        "riscv64" => {
            println!("cargo:rerun-if-changed=src/include/linkage.h");
            println!("cargo:rerun-if-changed=asm/riscv64/boot.s");
            println!("cargo:rerun-if-changed=asm/riscv64/linker.ld");
            println!("cargo:rustc-link-arg=-T{manifest_dir}/asm/riscv64/linker.ld");
            cc::Build::new()
                .file("asm/riscv64/boot.s")
                .include("src/include")
                .compile("startup");
        }
        _ => (),
    }
}
