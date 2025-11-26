pub struct CpuContext {
    pub rsp: u64,
    pub rip: u64,
    // регистры
}

pub fn init_tasks() {
    // создаем первый task (init)
}

pub fn switch_task(_from: &mut CpuContext, _to: &CpuContext) {
    // контекст-свитч
}
