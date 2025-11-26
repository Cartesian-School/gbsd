# GBSD – Full Project Structure (ASCII)

```shell
GBSD/
├── LICENSE
├── README.md
├── Makefile
├── Cargo.toml
├── build.rs
│
├── docs/
│   ├── architecture/
│   │   ├── microkernel-overview.md
│   │   ├── memory-model.md
│   │   ├── ipc-capabilities.md
│   │   ├── system-services.md
│   │   └── scheduler-design.md
│   ├── api/
│   │   ├── syscalls.md
│   │   ├── capabilities.md
│   │   ├── fs-api.md
│   │   ├── net-api.md
│   │   └── wayland-extensions.md
│   └── development/
│       ├── coding-style.md
│       ├── contributing.md
│       └── build-internals.md
│
├── kernel/
│   ├── arch/
│   │   ├── x86_64/
│   │   │   ├── boot/
│   │   │   │   ├── multiboot_header.S
│   │   │   │   ├── bootloader.rs
│   │   │   │   └── paging_init.rs
│   │   │   ├── interrupt/
│   │   │   │   ├── idt.rs
│   │   │   │   ├── isr.rs
│   │   │   │   └── irq.rs
│   │   │   └── mmu/
│   │   │       ├── page_table.rs
│   │   │       ├── frame_allocator.rs
│   │   │       └── tlb.rs
│   │   └── arm64/
│   │       ├── boot/
│   │       ├── mmu/
│   │       └── interrupt/
│   │
│   ├── core/
│   │   ├── scheduler/
│   │   │   ├── scheduler.rs
│   │   │   ├── queues.rs
│   │   │   └── context_switch.rs
│   │   ├── memory/
│   │   │   ├── vm.rs
│   │   │   ├── allocator.rs
│   │   │   ├── slab.rs
│   │   │   └── kheap.rs
│   │   ├── ipc/
│   │   │   ├── capability.rs
│   │   │   ├── msg_port.rs
│   │   │   ├── ipc_router.rs
│   │   │   └── serializers.rs
│   │   ├── fs/
│   │   │   ├── vfs.rs
│   │   │   ├── inode.rs
│   │   │   ├── path.rs
│   │   │   └── devfs.rs
│   │   ├── net/
│   │   │   ├── net_syscalls.rs
│   │   │   ├── socket_table.rs
│   │   │   └── tcp_state_machine.rs
│   │   ├── drivers/
│   │   │   ├── pci.rs
│   │   │   ├── ahci.rs
│   │   │   ├── e1000.rs
│   │   │   └── framebuffer.rs
│   │   ├── syscalls/
│   │   │   ├── table.rs
│   │   │   ├── process.rs
│   │   │   ├── thread.rs
│   │   │   ├── ipc.rs
│   │   │   ├── fs.rs
│   │   │   ├── net.rs
│   │   │   └── time.rs
│   │   └── kernel_main.rs
│   │
│   └── lib/
│       ├── kalloc/
│       ├── klog.rs
│       ├── sync/
│       │   ├── spinlock.rs
│       │   ├── mutex.rs
│       │   └── rwlock.rs
│       └── utils.rs
│
├── userspace/
│   ├── init/
│   │   ├── init.rs
│   │   └── service_manager.rs
│   │
│   ├── servers/
│   │   ├── fsd/                 # Filesystem server
│   │   │   ├── main.rs
│   │   │   ├── fs_impl.rs
│   │   │   └── journal.rs
│   │   ├── netd/                # Network stack server
│   │   │   ├── main.rs
│   │   │   ├── tcp.rs
│   │   │   ├── udp.rs
│   │   │   └── arp.rs
│   │   ├── logd/
│   │   │   ├── main.rs
│   │   │   └── ring_buffer.rs
│   │   ├── timed/
│   │   │   ├── main.rs
│   │   │   └── timers.rs
│   │   ├── windowd/             # Wayland
│   │   │   ├── main.rs
│   │   │   ├── compositor.rs
│   │   │   └── protocol/
│   │   │       └── gbsd_wl_ext.xml
│   │   └── inputd/
│   │       ├── main.rs
│   │       └── evdev.rs
│   │
│   ├── apps/
│   │   ├── shell/
│   │   │   ├── main.rs
│   │   │   └── parser.rs
│   │   ├── editor/
│   │   └── demo/
│   │
│   └── libc/                     # minimal C ABI layer
│       ├── string.c
│       ├── stdio.c
│       ├── syscall.c
│       └── include/
│           ├── stdio.h
│           ├── stdlib.h
│           ├── unistd.h
│           └── gbsd_sys.h
│
├── toolchain/
│   ├── linker.ld
│   ├── gbsd-gcc-wrapper.sh
│   ├── rust-target-spec.json
│   └── cargo-config.toml
│
├── scripts/
│   ├── build_iso.sh
│   ├── format_code.sh
│   ├── ci-checks.sh
│   ├── qemu_run.sh
│   └── debug_symbols.sh
│
├── build/
│   ├── iso/
│   ├── kernel/
│   ├── userspace/
│   └── logs/
│
├── tests/
│   ├── kernel/
│   │   ├── test_ipc.rs
│   │   ├── test_scheduler.rs
│   │   ├── test_memory.rs
│   │   └── test_syscalls.rs
│   ├── servers/
│   ├── apps/
│   └── integration/
│       ├── boot_test.rs
│       ├── fs_end_to_end.rs
│       └── net_stack.rs
│
└── ci/
    ├── github/
    │   └── workflows/
    │       ├── build.yml
    │       ├── test.yml
    │       └── release.yml
    ├── gitlab/
    │   └── .gitlab-ci.yml
    └── docker/
        ├── build-env.Dockerfile
        └── test-env.Dockerfile

```