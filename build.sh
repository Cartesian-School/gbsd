#!/bin/bash
set -e

# 1. Сборка ядра + servers
cargo build --release

# 2. Копируем бинарники в iso/root
mkdir -p iso/root
cp target/release/kernel iso/root/
cp -r target/release/servers iso/root/
cp -r target/release/apps iso/root/

# 3. Создаём ISO
grub-mkrescue -o gbsd-1.0.0-25-amd64.iso iso/

echo "✅ GBSD ISO built successfully!"
