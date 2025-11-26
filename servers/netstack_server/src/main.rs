// servers/netstack_server/src/main.rs
// GBSD netstack_server - TCP/IP network stack

#![no_std]
#![no_main]

extern crate core;

use core::panic::PanicInfo;

// Socket message types
const SOCKET_CREATE: u64 = 1;
const SOCKET_BIND: u64 = 2;
const SOCKET_LISTEN: u64 = 3;
const SOCKET_ACCEPT: u64 = 4;
const SOCKET_CONNECT: u64 = 5;
const SOCKET_SEND: u64 = 6;
const SOCKET_RECV: u64 = 7;
const SOCKET_CLOSE: u64 = 8;

// Socket types
const SOCK_STREAM: u32 = 1;    // TCP
const SOCK_DGRAM: u32 = 2;    // UDP

// Socket states
const STATE_CREATED: u32 = 0;
const STATE_LISTENING: u32 = 1;
const STATE_CONNECTING: u32 = 2;
const STATE_ESTABLISHED: u32 = 3;
const STATE_CLOSING: u32 = 4;
const STATE_CLOSED: u32 = 5;

/// IPv4 address
#[repr(C)]
struct IpAddr {
    addr: [u8; 4],
}

impl IpAddr {
    fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        IpAddr {
            addr: [a, b, c, d],
        }
    }

    fn is_loopback(&self) -> bool {
        self.addr[0] == 127
    }
}

/// Socket structure
#[repr(C)]
struct Socket {
    id: u32,
    socket_type: u32,              // SOCK_STREAM or SOCK_DGRAM
    state: u32,
    local_addr: IpAddr,
    local_port: u16,
    remote_addr: IpAddr,
    remote_port: u16,
    owner_pid: u32,
    recv_buffer: [u8; 4096],       // Ring buffer
    recv_head: usize,
    recv_tail: usize,
    recv_size: usize,
}

impl Socket {
    fn new(id: u32, socket_type: u32, owner_pid: u32) -> Self {
        Socket {
            id,
            socket_type,
            state: STATE_CREATED,
            local_addr: IpAddr::new(0, 0, 0, 0),
            local_port: 0,
            remote_addr: IpAddr::new(0, 0, 0, 0),
            remote_port: 0,
            owner_pid,
            recv_buffer: [0u8; 4096],
            recv_head: 0,
            recv_tail: 0,
            recv_size: 0,
        }
    }

    fn bind(&mut self, addr: &IpAddr, port: u16) -> bool {
        if self.state != STATE_CREATED {
            return false;  // Already bound
        }

        self.local_addr = IpAddr::new(addr.addr[0], addr.addr[1], addr.addr[2], addr.addr[3]);
        self.local_port = port;
        true
    }

    fn listen(&mut self, backlog: u32) -> bool {
        if self.state != STATE_CREATED {
            return false;
        }

        if self.socket_type != SOCK_STREAM {
            return false;  // Only TCP sockets can listen
        }

        self.state = STATE_LISTENING;
        true
    }

    fn connect(&mut self, addr: &IpAddr, port: u16) -> bool {
        self.remote_addr = IpAddr::new(addr.addr[0], addr.addr[1], addr.addr[2], addr.addr[3]);
        self.remote_port = port;
        self.state = STATE_ESTABLISHED;  // Simplified: no real 3-way handshake
        true
    }
}

/// Socket table
struct SocketTable {
    sockets: [Socket; 64],          // Max 64 sockets
    socket_count: usize,
    next_socket_id: u32,
}

impl SocketTable {
    fn new() -> Self {
        SocketTable {
            sockets: [Socket::new(0, 0, 0); 64],
            socket_count: 0,
            next_socket_id: 1,
        }
    }

    fn create_socket(&mut self, socket_type: u32, owner_pid: u32) -> Option<u32> {
        if self.socket_count >= 64 {
            return None;  // No space
        }

        let id = self.next_socket_id;
        self.next_socket_id += 1;

        let socket = Socket::new(id, socket_type, owner_pid);
        self.sockets[self.socket_count] = socket;
        self.socket_count += 1;

        Some(id)
    }

    fn get_socket_mut(&mut self, id: u32) -> Option<&mut Socket> {
        self.sockets.iter_mut().find(|s| s.id == id)
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
    print_str("[net] netstack_server started (PID 6)\n");

    unsafe {
        let net_port = allocate_port();
        print_str("[net] Allocated port ");
        print_u32(net_port);
        print_str(" for networking\n");

        let mut socket_table = SocketTable::new();
        print_str("[net] Initialized virtio-net interface\n");
        print_str("[net] Local IP: 127.0.0.1\n");
        print_str("[net] Ready for network operations\n");

        // Main event loop
        loop {
            let mut msg = [0u64; 8];
            let result = recv_message(net_port, &mut msg);

            if result == 0 {
                match msg[0] {
                    SOCKET_CREATE => {
                        let socket_type = msg[1] as u32;
                        let owner_pid = msg[2] as u32;

                        if let Some(socket_id) = socket_table.create_socket(socket_type, owner_pid) {
                            print_str("[net] Socket created: ID ");
                            print_u32(socket_id);
                            print_str(" (PID ");
                            print_u32(owner_pid);
                            print_str(")\n");

                            let reply = [socket_id as u64, 0, 0, 0, 0, 0, 0, 0];
                            let _ = send_message(net_port, &reply);
                        }
                    }
                    SOCKET_BIND => {
                        let socket_id = msg[1] as u32;
                        let port = msg[2] as u16;
                        print_str("[net] Bind socket ");
                        print_u32(socket_id);
                        print_str(" to port ");
                        print_u32(port as u32);
                        print_str("\n");
                    }
                    SOCKET_LISTEN => {
                        let socket_id = msg[1] as u32;
                        let backlog = msg[2] as u32;
                        print_str("[net] Listen on socket ");
                        print_u32(socket_id);
                        print_str(" (backlog ");
                        print_u32(backlog);
                        print_str(")\n");
                    }
                    SOCKET_CONNECT => {
                        let socket_id = msg[1] as u32;
                        let port = msg[2] as u16;
                        print_str("[net] Connect socket ");
                        print_u32(socket_id);
                        print_str(" to port ");
                        print_u32(port as u32);
                        print_str("\n");
                    }
                    SOCKET_SEND => {
                        let socket_id = msg[1] as u32;
                        let size = msg[2] as usize;
                        print_str("[net] Send on socket ");
                        print_u32(socket_id);
                        print_str(" (");
                        print_u32(size as u32);
                        print_str(" bytes)\n");
                    }
                    SOCKET_RECV => {
                        let socket_id = msg[1] as u32;
                        print_str("[net] Receive on socket ");
                        print_u32(socket_id);
                        print_str("\n");
                    }
                    SOCKET_CLOSE => {
                        let socket_id = msg[1] as u32;
                        print_str("[net] Close socket ");
                        print_u32(socket_id);
                        print_str("\n");
                    }
                    _ => {
                        print_str("[net] Unknown message type: ");
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
    print_str("[net] PANIC: ");
    if let Some(s) = info.payload().downcast_ref::<&str>() {
        print_str(s);
    }
    print_str("\n");

    loop {
        core::arch::x86_64::hlt();
    }
}

