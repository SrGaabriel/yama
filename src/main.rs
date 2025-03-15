fn main() {
    let uefi = true;
    let mut cmd = std::process::Command::new("C:/Program Files/qemu/qemu-system-x86_64");
    cmd.arg("-machine").arg("accel=whpx,type=q35");
    if uefi {
        let uefi_path = env!("UEFI_PATH");
        println!("Running UEFI at {}", uefi_path);
        cmd.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
        cmd.arg("-drive").arg(format!("format=raw,file={uefi_path}"));
        println!("QEMU: {:?}", cmd);
    } else {
        let bios_path = env!("BIOS_PATH");
        cmd.arg("-drive").arg(format!("format=raw,file={bios_path}"));
    }
    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap();
}