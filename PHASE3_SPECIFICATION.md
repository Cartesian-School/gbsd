# GBSD Phase 3: Core Services Implementation Specification

**Timeline**: Weeks 5-8 (4 weeks)  
**Status**: Ready to implement  
**Owner**: Development Team

---

## Phase 3 Overview

Phase 3 implements three critical core services that enable persistence, filesystem support, and networking:

1. **vfs_server** (Weeks 5-6) - Virtual filesystem abstraction + tmpfs
2. **ext4_server** (Week 7) - Persistent ext4 filesystem support
3. **netstack_server** (Week 8) - TCP/IP networking stack

By end of Phase 3, we want:
- ✅ File operations working (open, read, write, close)
- ✅ Filesystem persistence (ext4)
- ✅ Networking functional (TCP/UDP)
- ✅ POSIX socket API available
- ✅ Multiple filesystems supported

---

## vfs_server Specification (Weeks 5-6)

### Purpose
- Virtual filesystem abstraction layer
- Multiple filesystem support
- tmpfs (in-memory filesystem)
- File operations (open, read, write, close, seek)
- Directory operations (mkdir, rmdir, readdir, etc.)

### Architecture

```
Application
    ↓ (syscalls)
VFS Server
├─ tmpfs (in-memory)
├─ ext4 (disk-based, Phase 3.2)
└─ Future: NTFS, FAT32, etc.
    ↓ (syscalls)
Kernel (manage pages)
```

### Data Structures

```rust
// File node
struct INode {
    id: u64,
    mode: u32,           // S_IFREG | S_IFDIR | S_IFLNK
    uid: u32,
    gid: u32,
    size: u64,
    access_time: u64,
    modify_time: u64,
    change_time: u64,
    links: u32,
    file_type: FileType,
}

enum FileType {
    Regular,
    Directory,
    Symlink,
}

// Open file handle
struct FileHandle {
    inode_id: u64,
    flags: u32,           // O_RDONLY, O_WRONLY, O_RDWR, etc.
    offset: u64,          // Current read/write position
    owner_pid: u32,
}

// tmpfs storage
struct TmpfsVolume {
    inodes: HashMap<u64, INode>,
    data: Vec<u8>,       // All file data
    next_inode_id: u64,
}
```

### Syscall Interface

```
SYS_VFS_OPEN(path, flags, mode)   → file_descriptor
SYS_VFS_CLOSE(fd)                 → result
SYS_VFS_READ(fd, buf, count)      → bytes_read
SYS_VFS_WRITE(fd, buf, count)     → bytes_written
SYS_VFS_SEEK(fd, offset, whence)  → new_offset
SYS_VFS_STAT(path)                → stat_buffer
SYS_VFS_MKDIR(path, mode)         → result
SYS_VFS_RMDIR(path)               → result
SYS_VFS_READDIR(fd)               → directory_entry
SYS_VFS_UNLINK(path)              → result
SYS_VFS_SYMLINK(target, link)     → result
SYS_VFS_READLINK(path)            → target_path
SYS_VFS_MOUNT(device, path, fs)   → result
SYS_VFS_UMOUNT(path)              → result
```

### Implementation Steps

**Week 5 (tmpfs in-memory)**
1. Create INode and FileHandle structures
2. Implement tmpfs volume management
3. Implement file creation and deletion
4. Implement file read/write
5. Implement directory operations

**Week 6 (VFS abstraction)**
1. Create VFS abstraction layer
2. Implement syscall handlers
3. Implement file descriptor table
4. Implement mount system
5. Add basic ACLs

### Target Output

```
[vfs] vfs_server started (PID X)
[vfs] Allocated port for VFS
[vfs] tmpfs mounted at /
[vfs] Ready for file operations
```

---

## ext4_server Specification (Week 7)

### Purpose
- Persistent ext4 filesystem support
- Block device I/O
- Journal support (JBD2)
- Block caching
- Inode caching

### Architecture

```
vfs_server (requests)
    ↓
ext4_server
├─ Block cache
├─ Inode cache
├─ Journal (JBD2)
└─ ext4 metadata parser
    ↓ (block I/O)
Disk (via QEMU block device)
```

### Key Components

```rust
// Block cache
struct BlockCache {
    cache: LRU<u64, Block>,     // block_id -> data
    dirty: HashSet<u64>,        // dirty blocks
    max_size: usize,            // entries
}

// ext4 superblock
#[repr(C)]
struct Ext4Superblock {
    inode_count: u32,
    block_count: u32,
    reserved_blocks: u32,
    free_blocks: u32,
    free_inodes: u32,
    block_size: u32,
    fragment_size: i32,
    blocks_per_group: u32,
    fragments_per_group: u32,
    inodes_per_group: u32,
    mount_time: u32,
    write_time: u32,
    mount_count: u16,
    max_mount_count: i16,
    magic: u16,
    state: u16,
    errors: u16,
    // ... more fields
}

// ext4 block group descriptor
#[repr(C)]
struct Ext4BlockGroupDescriptor {
    block_bitmap: u32,
    inode_bitmap: u32,
    inode_table: u32,
    free_blocks: u16,
    free_inodes: u16,
    used_dirs: u16,
    // ... checksum field
}
```

### Implementation Steps

1. Parse ext4 superblock
2. Read block group descriptors
3. Implement block reading (with cache)
4. Implement inode reading/writing
5. Implement directory scanning
6. Implement file extent mapping
7. Implement block allocation
8. Add journaling support

### Messages

```
SYS_BLOCK_READ(device, block_id, buffer)   → bytes_read
SYS_BLOCK_WRITE(device, block_id, buffer)  → bytes_written
SYS_BLOCK_SYNC(device)                     → result
```

### Target Output

```
[ext4] ext4_server started (PID X)
[ext4] Allocated port for ext4
[ext4] Found ext4 filesystem at /dev/disk0
[ext4] Block size: 4096 bytes
[ext4] Total inodes: XXXXX
[ext4] Ready for filesystem operations
```

---

## netstack_server Specification (Week 8)

### Purpose
- TCP/IP protocol stack
- UDP support
- Socket API (POSIX-like)
- Ethernet layer (QEMU virtio-net)
- IP routing (basic)
- ARP support

### Architecture

```
Applications (sockets)
    ↓
Socket API
    ↓
netstack_server
├─ TCP/IP stack
├─ UDP stack
├─ ARP cache
├─ Routing table
└─ Network interfaces
    ↓ (virtio-net)
QEMU Virtual NIC
```

### Protocol Stack

```
Layer 4: TCP/UDP
    ↓
Layer 3: IP (IPv4)
    ↓
Layer 2: Ethernet + ARP
    ↓
Layer 1: virtio-net (QEMU)
```

### Data Structures

```rust
// Socket
struct Socket {
    id: u32,
    socket_type: SocketType,     // SOCK_STREAM, SOCK_DGRAM
    state: SocketState,          // LISTEN, ESTABLISHED, etc.
    local_addr: IpAddr,
    local_port: u16,
    remote_addr: IpAddr,
    remote_port: u16,
    recv_buffer: RingBuffer,
    send_buffer: RingBuffer,
    owner_pid: u32,
}

enum SocketType {
    Stream,      // TCP
    Datagram,    // UDP
}

enum SocketState {
    Created,
    Listening,
    Connecting,
    Established,
    Closing,
    Closed,
}

// IP address
#[repr(C)]
struct IpAddr {
    addr: [u8; 4],    // IPv4 in network byte order
}

// TCP header
#[repr(C)]
struct TcpHeader {
    src_port: u16,
    dst_port: u16,
    seq_num: u32,
    ack_num: u32,
    flags: u8,         // SYN, ACK, FIN, RST, etc.
    window_size: u16,
    checksum: u16,
    urgent: u16,
}
```

### Socket Syscalls

```
SYS_SOCKET(domain, type, protocol)       → socket_fd
SYS_BIND(fd, addr, addr_len)             → result
SYS_LISTEN(fd, backlog)                  → result
SYS_ACCEPT(fd)                           → client_fd
SYS_CONNECT(fd, addr, addr_len)          → result
SYS_SEND(fd, buf, len, flags)            → bytes_sent
SYS_RECV(fd, buf, len, flags)            → bytes_received
SYS_SENDTO(fd, buf, len, addr)           → bytes_sent
SYS_RECVFROM(fd, buf, len)               → (bytes, sender_addr)
SYS_CLOSE(fd)                            → result
SYS_SETSOCKOPT(fd, level, optname, val)  → result
SYS_GETSOCKOPT(fd, level, optname)       → value
```

### Implementation Steps

**Week 8 (Basic TCP/UDP)**
1. Initialize network interfaces
2. Implement Ethernet framing
3. Implement ARP protocol
4. Implement IPv4 routing
5. Implement UDP stack
6. Implement TCP state machine (SYN, ACK, etc.)
7. Implement socket API
8. Add loopback support

### Target Output

```
[net] netstack_server started (PID X)
[net] Allocated port for networking
[net] Initialized virtio-net interface
[net] Local IP: 192.168.0.X
[net] Gateway: 192.168.0.1
[net] Ready for network operations
```

---

## IPC Communication Patterns (Phase 3)

### vfs_server ↔ Kernel
```
Application
    ↓ (SYS_VFS_OPEN)
Kernel
    ↓ (message to vfs_server)
vfs_server processes request
    ↓ (reply with file_descriptor)
Application
```

### ext4_server ↔ vfs_server
```
vfs_server (read inode from disk)
    ↓ (message: "read block X")
ext4_server
    ↓ (read from disk)
    ↓ (reply: block data)
vfs_server (caches result)
```

### netstack_server ↔ Application
```
Application (socket syscall)
    ↓
Kernel (creates socket)
    ↓ (message to netstack_server)
netstack_server (manages socket)
    ↓ (network packet arrives)
    ↓ (message to application)
Application (receives data)
```

---

## Service Dependencies

```
Applications
    ↓
vfs_server (needs both)
├─ ext4_server (for disk)
└─ netstack_server (for network access)

Both depend on:
├─ init_server (bootstrap)
├─ log_server (logging)
└─ scheduler_server (scheduling)

All depend on:
└─ Kernel (10 syscalls + new filesystem/socket syscalls)
```

---

## Testing During Phase 3

### Unit Tests
```rust
#[test]
fn test_tmpfs_file_create() {
    // Create file in tmpfs
}

#[test]
fn test_ext4_inode_read() {
    // Read inode from ext4
}

#[test]
fn test_socket_creation() {
    // Create TCP/UDP socket
}
```

### Integration Tests
```bash
# Test file I/O
touch /tmp/test.txt
echo "hello" > /tmp/test.txt
cat /tmp/test.txt

# Test TCP connection
nc -l 0.0.0.0 9999 &
nc 127.0.0.1 9999
```

### System Tests (QEMU)
```bash
# Boot with persistent disk
qemu-system-x86_64 -kernel kernel -drive file=disk.img,if=virtio

# Test filesystem operations
# Test network loopback
# Test combined operations
```

---

## Success Criteria

### vfs_server (End of Week 6)
- ✅ tmpfs working end-to-end
- ✅ File operations (open, read, write, close)
- ✅ Directory operations
- ✅ Mount system working
- ✅ Multiple filesystems can coexist

### ext4_server (End of Week 7)
- ✅ ext4 disk reading
- ✅ Inode caching
- ✅ Block caching
- ✅ File persistence
- ✅ Journal support

### netstack_server (End of Week 8)
- ✅ Socket API working
- ✅ TCP connections possible
- ✅ UDP datagrams work
- ✅ Loopback networking
- ✅ Basic routing

### Phase 3 Overall (End of Week 8)
- ✅ All three services functional
- ✅ File I/O working
- ✅ Network working
- ✅ Integration tested
- ✅ Ready for Phase 4

---

## Resource Requirements

### Memory
- tmpfs: Up to 256 MB (configurable)
- Block cache: 16 MB
- Socket buffers: 64 KB per socket

### Disk
- ext4 filesystem: Test disk ~100 MB
- Journal: Embedded in ext4

### Network
- Virtual network interface (QEMU)
- 10 Mbps bandwidth (simulated)

---

## Performance Targets

| Operation | Target | Metric |
|-----------|--------|--------|
| File read | < 100 µs | Per operation |
| File write | < 100 µs | Per operation |
| TCP latency | < 1 ms | One-way |
| UDP latency | < 500 µs | One-way |
| Throughput | > 10 Mbps | Sustained |

---

## Documentation Required

For each service:
1. Architecture diagram
2. Syscall reference
3. Implementation guide
4. Testing procedures
5. Performance baseline

---

**Status**: Ready for implementation  
**Effort**: 4 weeks  
**Timeline**: Weeks 5-8 of 12-week MVP  


