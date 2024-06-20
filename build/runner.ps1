#!/usr/bin/env pwsh
$KERNEL_BIN = $args[0]
$LIMINE_GIT = 'https://github.com/limine-bootloader/limine.git'
$TARGET_PATH = 'target/limine'
$LIMINE_PATH = 'target/limine/bootloader'

New-Item $TARGET_PATH -ItemType Directory -ErrorAction SilentlyContinue

if (!(Test-Path -Path $LIMINE_PATH)) {
    git clone $LIMINE_GIT --depth=1 --branch v3.0-branch-binary $LIMINE_PATH
}

Push-Location $LIMINE_PATH
git pull
make
Pop-Location

Copy-Item $KERNEL_BIN, ./build/limine.cfg, $LIMINE_PATH/limine.sys, $LIMINE_PATH/limine-cd.bin, $LIMINE_PATH/limine-cd-efi.bin $TARGET_PATH

# TODO: make xplat - remove xorriso
xorriso -as mkisofs `
    -b limine-cd.bin `
    -no-emul-boot -boot-load-size 4 -boot-info-table `
    --efi-boot limine-cd-efi.bin `
    -efi-boot-part --efi-boot-image --protective-msdos-label `
    $TARGET_PATH -o "$TARGET_PATH/os.iso"

&"$LIMINE_PATH/limine-deploy" $TARGET_PATH/os.iso

if ($null -eq $env:QEMU) {
    $QEMU = 'qemu-system-x86_64'
} else {
    $QEMU = $env:QEMU
}

&$QEMU `
    -machine q35 -cpu qemu64 -M smm=off `
    -no-reboot -serial stdio -cdrom "$TARGET_PATH/os.iso" -d int,cpu_reset $args
