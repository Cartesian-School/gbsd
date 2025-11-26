// servers/scheduler_server/src/main.rs
// GBSD scheduler_server - preemptive scheduling

#![no_std]
#![no_main]

extern crate core;

use core::panic::PanicInfo;

// Scheduler message types
const MSG_TIMER_TICK: u64 = 1;
const MSG_TASK_YIELD: u64 = 2;
const MSG_TASK_SLEEP: u64 = 3;

/// Scheduler state
struct Scheduler {
    ready_queue: [u32; 256],      // PIDs of ready tasks
    queue_head: usize,
    queue_tail: usize,
    queue_size: usize,

    sleeping: [(u32, u64); 256],  // (PID, wake_time)
    sleeping_count: usize,

    current_pid: u32,
}

impl Scheduler {
    fn new() -> Self {
        Scheduler {
            ready_queue: [0u32; 256],
            queue_head: 0,
            queue_tail: 0,
            queue_size: 0,
            sleeping: [(0, 0); 256],
            sleeping_count: 0,
            current_pid: 1,  // Start with init_server
        }
    }

    fn enqueue(&mut self, pid: u32) {
        if self.queue_size < 256 {
            self.ready_queue[self.queue_tail] = pid;
            self.queue_tail = (self.queue_tail + 1) % 256;
            self.queue_size += 1;
        }
    }

    fn dequeue(&mut self) -> Option<u32> {
        if self.queue_size > 0 {
            let pid = self.ready_queue[self.queue_head];
            self.queue_head = (self.queue_head + 1) % 256;
            self.queue_size -= 1;
            Some(pid)
        } else {
            None
        }
    }

    fn wake_expired_tasks(&mut self, now: u64) {
        let mut i = 0;
        while i < self.sleeping_count {
            if self.sleeping[i].1 <= now {
                self.enqueue(self.sleeping[i].0);
                // Swap with last
                self.sleeping[i] = self.sleeping[self.sleeping_count - 1];
                self.sleeping_count -= 1;
            } else {
                i += 1;
            }
        }
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

/// Get current time via syscall
unsafe fn sys_time() -> u64 {
    let result: u64;
    core::arch::x86_64::asm!(
        "syscall",
        inout("rax") 10u64 => result,  // SYS_TIME = 10
    );
    result
}

/// Switch to target process via syscall
unsafe fn sched_switch(target_pid: u32) -> u64 {
    let result: u64;
    core::arch::x86_64::asm!(
        "syscall",
        inout("rax") 9u64 => result,  // SYS_SCHED_SWITCH = 9
        in("rdi") target_pid as u64,
    );
    result
}

/// Main entry point
#[no_mangle]
pub extern "C" fn _start() -> ! {
    print_str("[scheduler] scheduler_server started (PID 3)\n");

    unsafe {
        let sched_port = allocate_port();
        print_str("[scheduler] Allocated port ");
        print_u32(sched_port);
        print_str(" for scheduling\n");

        let mut scheduler = Scheduler::new();

        print_str("[scheduler] Ready queue: empty\n");
        print_str("[scheduler] Waiting for events...\n");

        // Main loop
        loop {
            // Wake any expired sleeping tasks
            let now = sys_time();
            scheduler.wake_expired_tasks(now);

            // Try to receive a message
            let mut msg = [0u64; 8];
            let result = recv_message(sched_port, &mut msg);

            if result == 0 {
                match msg[0] {
                    MSG_TIMER_TICK => {
                        // Put current task back in queue
                        if scheduler.current_pid != 0 {
                            scheduler.enqueue(scheduler.current_pid);
                        }

                        // Pick next from queue
                        if let Some(next) = scheduler.dequeue() {
                            scheduler.current_pid = next;
                            let _ = sched_switch(next);
                        }
                    }
                    MSG_TASK_YIELD => {
                        let yielding_pid = msg[1] as u32;

                        // Put yielding task back in queue
                        scheduler.enqueue(yielding_pid);

                        // Pick next
                        if let Some(next) = scheduler.dequeue() {
                            scheduler.current_pid = next;
                            let _ = sched_switch(next);
                        }
                    }
                    MSG_TASK_SLEEP => {
                        let pid = msg[1] as u32;
                        let duration = msg[2];

                        // Add to sleeping map
                        let wake_time = sys_time() + duration;
                        if scheduler.sleeping_count < 256 {
                            scheduler.sleeping[scheduler.sleeping_count] = (pid, wake_time);
                            scheduler.sleeping_count += 1;
                        }

                        // Switch to next ready task
                        if let Some(next) = scheduler.dequeue() {
                            scheduler.current_pid = next;
                            let _ = sched_switch(next);
                        }
                    }
                    _ => {
                        print_str("[scheduler] Unknown message: ");
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
    print_str("[scheduler] PANIC: ");
    if let Some(s) = info.payload().downcast_ref::<&str>() {
        print_str(s);
    }
    print_str("\n");

    loop {
        core::arch::x86_64::hlt();
    }
}

