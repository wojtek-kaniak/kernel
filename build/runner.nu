#! /usr/bin/env nu

def main [binary: path, ...args] {
    let limine_git = 'https://github.com/limine-bootloader/limine.git'
    let target_path = 'target/limine'
    let limine_path = 'target/limine/bootloader'

    mkdir $target_path

    if !($limine_path | path exists) {
        git clone $limine_git --depth=1 --branch v3.0-branch-binary $limine_path
    }

    enter $limine_path
    git pull
    make
    dexit

    (
    cp $binary build/limine.cfg $"($limine_path)/limine.sys"
        $"($limine_path)/limine-cd.bin" $"($limine_path)/limine-cd-efi.bin"
        $target_path
    )

    (
    # TODO: make xplat - remove xorriso
    xorriso -as mkisofs
        -b limine-cd.bin
        -no-emul-boot -boot-load-size 4 -boot-info-table
        --efi-boot limine-cd-efi.bin
        -efi-boot-part --efi-boot-image --protective-msdos-label
        $target_path -o $"($target_path)/os.iso"
    )

    ^$"($target_path)/limine-deploy" $"($target_path)/os.iso"

    let qemu = if $env.QEMU? == null {
        'qemu-system-x86_64'
    } else {
        $env.QEMU
    }

    (
    ^$"($qemu)"
        -machine q35 -cpu qemu64 -M smm=off
        -no-reboot -serial stdio -cdrom "$TARGET_PATH/os.iso" -d int,cpu_reset $args
    )
}
