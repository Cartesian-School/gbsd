#!/usr/bin/env python3
# Сборка ISO (build.py)
import subprocess
import os

os.makedirs("iso/boot/grub", exist_ok=True)

subprocess.run(["cargo", "build", "--release", "--target", "x86_64-gbsd.json"], check=True)

subprocess.run(["cp", "target/x86_64-gbsd/release/kernel", "iso/boot/"])

with open("iso/boot/grub/grub.cfg", "w") as f:
    f.write('''
menuentry "GBSD" {
    multiboot2 /boot/kernel
    boot
}
''')

subprocess.run(["grub-mkrescue", "-o", "gbsd.iso", "iso"])
print("gbsd.iso created!")