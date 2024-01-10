use std::process::Command;
use std::{env, fs, process};

fn main() {
    let current_exe = env::current_exe().unwrap();
    let uefi_target = current_exe.with_file_name("uefi.img");

    fs::copy(env!("UEFI_IMAGE"), &uefi_target).unwrap();

    println!("UEFI disk image at {}", uefi_target.display());

    let mut qemu = Command::new("qemu-system-x86_64");
    qemu.arg("-debugcon").arg("stdio");
    qemu.arg("-drive");
    qemu.arg(format!("format=raw,file={}", env!("UEFI_IMAGE")));
    qemu.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
    let exit_status = qemu.status().unwrap();
    process::exit(exit_status.code().unwrap_or(-1));
}
