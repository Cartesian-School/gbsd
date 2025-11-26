// servers/vfs_server/src/main.rs
// GBSD vfs_server - Virtual filesystem with tmpfs

#![no_std]
#![no_main]

extern crate core;

use core::panic::PanicInfo;

// VFS message types
const VFS_OPEN: u64 = 1;
const VFS_CLOSE: u64 = 2;
const VFS_READ: u64 = 3;
const VFS_WRITE: u64 = 4;
const VFS_SEEK: u64 = 5;
const VFS_STAT: u64 = 6;
const VFS_MKDIR: u64 = 7;
const VFS_RMDIR: u64 = 8;
const VFS_READDIR: u64 = 9;
const VFS_UNLINK: u64 = 10;

// File modes
const S_IFREG: u32 = 0o100000;  // Regular file
const S_IFDIR: u32 = 0o040000;  // Directory

/// INode representation
#[repr(C)]
struct INode {
    id: u64,
    mode: u32,              // File type and permissions
    size: u64,              // File size in bytes
    modify_time: u64,       // Last modification time
    owner_pid: u32,         // Owner process
    file_type: u8,          // 0=Regular, 1=Directory
}

impl INode {
    fn new(id: u64, mode: u32, owner_pid: u32) -> Self {
        INode {
            id,
            mode,
            size: 0,
            modify_time: 0,
            owner_pid,
            file_type: 0,
        }
    }
}

/// File handle
#[repr(C)]
struct FileHandle {
    inode_id: u64,
    flags: u32,             // O_RDONLY, O_WRONLY, O_RDWR
    offset: u64,            // Current position
    owner_pid: u32,
}

/// tmpfs volume storage
struct TmpfsVolume {
    inodes: [INode; 256],           // Max 256 inodes
    inode_count: usize,
    data: [u8; 65536],              // 64 KB data storage
    data_used: usize,
    next_inode_id: u64,
    file_handles: [FileHandle; 64], // Max 64 open files
    handle_count: usize,
}

impl TmpfsVolume {
    fn new() -> Self {
        let mut vol = TmpfsVolume {
            inodes: [INode {
                id: 0,
                mode: 0,
                size: 0,
                modify_time: 0,
                owner_pid: 0,
                file_type: 0,
            }; 256],
            inode_count: 0,
            data: [0u8; 65536],
            data_used: 0,
            next_inode_id: 1,
            file_handles: [FileHandle {
                inode_id: 0,
                flags: 0,
                offset: 0,
                owner_pid: 0,
            }; 64],
            handle_count: 0,
        };

        // Create root directory inode
        vol.inodes[0] = INode {
            id: 0,
            mode: S_IFDIR | 0o755,
            size: 0,
            modify_time: 0,
            owner_pid: 1,
            file_type: 1,  // directory
        };
        vol.inode_count = 1;

        vol
    }

    fn create_file(&mut self, mode: u32, owner_pid: u32) -> Option<u64> {
        if self.inode_count >= 256 {
            return None;  // No space for new inode
        }

        let id = self.next_inode_id;
        self.inode_count += 1;
        self.next_inode_id += 1;

        let idx = self.inode_count - 1;
        self.inodes[idx] = INode {
            id,
            mode: mode | S_IFREG,
            size: 0,
            modify_time: 0,
            owner_pid,
            file_type: 0,  // regular file
        };

        Some(id)
    }

    fn write_file(&mut self, inode_id: u64, data: &[u8]) -> Option<usize> {
        // Find inode
        let inode_idx = self.inodes.iter().position(|i| i.id == inode_id)?;
        let inode = &mut self.inodes[inode_idx];

        // Check space available
        let space_available = self.data.len() - self.data_used;
        let bytes_to_write = if data.len() <= space_available {
            data.len()
        } else {
            space_available
        };

        if bytes_to_write == 0 {
            return None;  // No space
        }

        // Copy data
        let start = self.data_used;
        for (i, &byte) in data[..bytes_to_write].iter().enumerate() {
            self.data[start + i] = byte;
        }

        self.data_used += bytes_to_write;
        inode.size = bytes_to_write as u64;

        Some(bytes_to_write)
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

/// Print a u64
fn print_u64(n: u64) {
    if n == 0 {
        print_str("0");
        return;
    }

    let mut buf = [0u8; 20];
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
    print_str("[vfs] vfs_server started (PID 4)\n");

    unsafe {
        let vfs_port = allocate_port();
        print_str("[vfs] Allocated port ");
        print_u64(vfs_port as u64);
        print_str(" for VFS\n");

        let mut tmpfs = TmpfsVolume::new();
        print_str("[vfs] tmpfs mounted at /\n");
        print_str("[vfs] Ready for file operations\n");

        // Main event loop
        loop {
            let mut msg = [0u64; 8];
            let result = recv_message(vfs_port, &mut msg);

            if result == 0 {
                match msg[0] {
                    VFS_OPEN => {
                        // Create a new file
                        if let Some(inode_id) = tmpfs.create_file(0o644, msg[3] as u32) {
                            // Send back the inode_id as file descriptor
                            let reply = [inode_id, 0, 0, 0, 0, 0, 0, 0];
                            let _ = send_message(vfs_port, &reply);
                        }
                    }
                    VFS_WRITE => {
                        // Write to a file
                        let inode_id = msg[1];
                        let size = msg[2] as usize;
                        // In real impl, would copy data from message
                        print_str("[vfs] Write request for inode ");
                        print_u64(inode_id);
                        print_str(" (");
                        print_u64(size as u64);
                        print_str(" bytes)\n");
                    }
                    VFS_READ => {
                        print_str("[vfs] Read request received\n");
                    }
                    VFS_STAT => {
                        print_str("[vfs] Stat request received\n");
                    }
                    _ => {
                        print_str("[vfs] Unknown message type: ");
                        print_u64(msg[0]);
                        print_str("\n");
                    }
                }
            }
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print_str("[vfs] PANIC: ");
    if let Some(s) = info.payload().downcast_ref::<&str>() {
        print_str(s);
    }
    print_str("\n");

    loop {
        core::arch::x86_64::hlt();
    }
}

