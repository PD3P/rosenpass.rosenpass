#!/usr/bin/env bash

set -euo pipefail
IFS=$'\n\t'

apt update
apt install -y --no-install-recommends libclang-dev cmake qemu-system-x86
rustup toolchain install nightly --component rust-src

# export CC_ENABLE_DEBUG_OUTPUT=1
export CC_x86_64_unknown_hermit=x86_64-hermit-gcc
export AR_x86_64_unknown_hermit=x86_64-hermit-gcc-ar
export CMAKE_TOOLCHAIN_FILE_x86_64_unknown_hermit=/mnt/rosenpass/hermit-toolchain.cmake
export BINDGEN_EXTRA_CLANG_ARGS="--sysroot=/opt/hermit/x86_64-hermit -I/opt/hermit/x86_64-hermit/include -I/opt/hermit/lib/gcc/x86_64-hermit/7.5.0/include -I/opt/hermit/lib/gcc/x86_64-hermit/7.5.0/include-fixed"

cargo +nightly build -Zbuild-std=std,panic_abort --target x86_64-unknown-hermit --package rosenpass --no-default-features

qemu-system-x86_64 \
    -enable-kvm \
    -cpu host \
    -smp 1 \
    -m 512 \
    -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
    -display none -serial stdio \
    -netdev user,id=u1,hostfwd=tcp::9975-:9975,hostfwd=udp::9975-:9975,net=192.168.76.0/24,dhcpstart=192.168.76.9 \
    -device virtio-net-pci,netdev=u1,disable-legacy=on,packed=on,mq=on \
    -kernel hermit-loader-x86_64 \
    -initrd target/x86_64-unknown-hermit/debug/rosenpass \
    -append "env=RUST_LOG=trace -- exchange-config /hermit-rosenpass-config.toml"
