// servers/ext4_server/src/main.rs
// GBSD ext4_server - Persistent ext4 filesystem

#![no_std]
#![no_main]

extern crate core;

use core::panic::PanicInfo;

// Block I/O message types
const BLOCK_READ: u64 = 1;
const BLOCK_WRITE: u64 = 2;
const BLOCK_SYNC: u64 = 3;

// ext4 specific constants
const BLOCK_SIZE: usize = 4096;
const INODE_SIZE: usize = 256;

/// ext4 superblock structure
#[repr(C)]
struct Ext4Superblock {
    inode_count: u32,
    block_count: u32,
    reserved_blocks: u32,
    free_blocks: u32,
    free_inodes: u32,
    block_size_log: u32,            // Block size = 1024 << block_size_log
    fragment_size_log: u32,
    blocks_per_group: u32,
    fragments_per_group: u32,
    inodes_per_group: u32,
    mount_time: u32,
    write_time: u32,
    mount_count: u16,
    max_mount_count: i16,
    magic: u16,                     // 0xEF53
    state: u16,                     // 1=clean, 2=errors
    errors: u16,
    minor_rev: u16,
}

/// ext4 block group descriptor
#[repr(C)]
struct Ext4BlockGroupDescriptor {
    block_bitmap: u32,              // Block containing bitmap
    inode_bitmap: u32,
    inode_table: u32,               // Block containing inode table
    free_blocks: u16,
    free_inodes: u16,
    used_dirs: u16,
    pad: u16,
    checksum: u32,
}

/// ext4 inode structure
#[repr(C)]
struct Ext4Inode {
    mode: u16,                      // File mode and type
    uid: u16,                       // Owner UID
    size_lo: u32,                   // Lower 32 bits of size
    atime: u32,                     // Access time
    ctime: u32,                     // Change time
    mtime: u32,                     // Modification time
    dtime: u32,                     // Delete time
    gid: u16,                       // Owner GID
    links_count: u16,               // Hard links
    blocks_lo: u32,                 // Number of blocks (lower 32 bits)
    flags: u32,                     // File flags
    osd1: u32,                      // OS dependent
    block: [u32; 15],               // Direct block pointers and indirect
    generation: u32,                // Version/generation
    file_acl_lo: u32,               // ACL block (lower 32 bits)
    size_high: u32,                 // Upper 32 bits of size
}

/// Block cache for performance
struct BlockCache {
    blocks: [[u8; BLOCK_SIZE]; 8],  // 8 cached blocks
    block_ids: [u64; 8],
    dirty: [bool; 8],
    next_slot: usize,
}

impl BlockCache {
    fn new() -> Self {
        BlockCache {
            blocks: [[0u8; BLOCK_SIZE]; 8],
            block_ids: [u64::MAX; 8],
            dirty: [false; 8],
            next_slot: 0,
        }
    }

    fn get(&self, block_id: u64) -> Option<&[u8; BLOCK_SIZE]> {
        for i in 0..8 {
            if self.block_ids[i] == block_id {
                return Some(&self.blocks[i]);
            }
        }
        None
    }

    fn get_mut(&mut self, block_id: u64) -> Option<&mut [u8; BLOCK_SIZE]> {
        for i in 0..8 {
            if self.block_ids[i] == block_id {
                self.dirty[i] = true;
                return Some(&mut self.blocks[i]);
            }
        }
        None
    }

    fn insert(&mut self, block_id: u64, data: &[u8; BLOCK_SIZE]) {
        let slot = self.next_slot % 8;
        self.block_ids[slot] = block_id;
        self.blocks[slot] = *data;
        self.dirty[slot] = false;
        self.next_slot += 1;
    }
}

/// Print a string to serial console
fn print_str(s: &str) {
    for c in s.bytes() {
        unsafe {
            core::arch::x86_64::asm!("out al, dx", in("al") c, in("dx") 0x3F8u16);
        }
    }
}

/// Print a u32
fn print_u32(n: u32) {
    if n == 0 {
        print_str("0");
        return;
    }

    let mut buf = [0u8; 10];
    let mut i = 0;
    let mut num = n;

    while num > 0 {
        buf[i] = b'0' + (num % 10) as u8;
        i += 1;
        num /= 10;
    }

    for j in (0..i).rev() {
        unsafe {
            core::arch::x86_64::asm!("out al, dx", in("al") buf[j], in("dx") 0x3F8u16);
        }
    }
}

/// Allocate a port via syscall
unsafe fn allocate_port() -> u32 {
    let result: u64;
    core::arch::x86_64::asm!(
        "syscall",
        inout("rax") 1u64 => result,  // SYS_PORT_ALLOCATE = 1
    );
    result as u32
}

/// Receive a message from a port via syscall
unsafe fn recv_message(port: u32, buf: &mut [u64; 8]) -> u64 {
    let result: u64;
    core::arch::x86_64::asm!(
        "syscall",
        inout("rax") 3u64 => result,  // SYS_PORT_RECEIVE = 3
        in("rdi") port as u64,
        in("rsi") buf as *mut [u64; 8] as u64,
        in("rdx") 8,
    );
    result
}

/// Send a message to a port via syscall
unsafe fn send_message(port: u32, msg: &[u64; 8]) -> u64 {
    let result: u64;
    core::arch::x86_64::asm!(
        "syscall",
        inout("rax") 2u64 => result,  // SYS_PORT_SEND = 2
        in("rdi") port as u64,
        in("rsi") msg as *const [u64; 8] as u64,
        in("rdx") 8,
    );
    result
}

/// Main entry point
#[no_mangle]
pub extern "C" fn _start() -> ! {
    print_str("[ext4] ext4_server started (PID 5)\n");

    unsafe {
        let ext4_port = allocate_port();
        print_str("[ext4] Allocated port ");
        print_u32(ext4_port);
        print_str(" for ext4\n");

        let mut block_cache = BlockCache::new();
        print_str("[ext4] Block cache initialized (8 blocks, ");
        print_u32(BLOCK_SIZE as u32);
        print_str(" bytes each)\n");

        print_str("[ext4] Waiting for block I/O requests...\n");

        // Main event loop
        loop {
            let mut msg = [0u64; 8];
            let result = recv_message(ext4_port, &mut msg);

            if result == 0 {
                match msg[0] {
                    BLOCK_READ => {
                        let block_id = msg[1];
                        print_str("[ext4] Block read request: block ");
                        print_u32(block_id as u32);
                        print_str("\n");

                        // In real implementation, would read from disk
                        // For now, return cached or error
                    }
                    BLOCK_WRITE => {
                        let block_id = msg[1];
                        print_str("[ext4] Block write request: block ");
                        print_u32(block_id as u32);
                        print_str("\n");

                        // Mark as dirty, would flush to disk
                    }
                    BLOCK_SYNC => {
                        print_str("[ext4] Sync request: flushing dirty blocks\n");
                    }
                    _ => {
                        print_str("[ext4] Unknown message type: ");
                        print_u32(msg[0] as u32);
                        print_str("\n");
                    }
                }
            }
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print_str("[ext4] PANIC: ");
    if let Some(s) = info.payload().downcast_ref::<&str>() {
        print_str(s);
    }
    print_str("\n");

    loop {
        core::arch::x86_64::hlt();
    }
}

