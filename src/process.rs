use libc::{mmap, MAP_ANONYMOUS, MAP_PRIVATE, PROT_READ, PROT_WRITE, MAP_FAILED};
use std::ptr;

pub unsafe fn start(entry: u64, phdr_addr: u64) {
    let stack_size = 0x10000;

    let stack = unsafe {
        mmap(
            ptr::null_mut(),
            stack_size,
            PROT_READ | PROT_WRITE,
            MAP_PRIVATE | MAP_ANONYMOUS,
            -1,
            0,
        )
    };
    if stack == MAP_FAILED {
        panic!("stack mapping failed");
    }

    // Write "paasa" string into the bottom of the stack page and forget it —
    // we never return from the asm jump so we manage this memory manually.
    let prog_str = b"paasa\0";
    let str_dst = stack as *mut u8;
    unsafe { ptr::copy_nonoverlapping(prog_str.as_ptr(), str_dst, prog_str.len()); }
    let prog_ptr = str_dst as u64;

    // sp starts at the top of the stack (high address), we push downward.
    let mut sp = unsafe { (stack as *mut u8).add(stack_size) as *mut u64 };

    macro_rules! push {
        ($val:expr) => {
            unsafe {
                sp = sp.offset(-1);
                *sp = $val as u64;
            }
        };
    }

    // auxv — written in reverse (last entry first, kernel reads low→high)
    push!(0u64);        // AT_NULL value
    push!(0u64);        // AT_NULL type
    push!(4096u64);     // AT_PAGESZ value
    push!(6u64);        // AT_PAGESZ type
    push!(entry);       // AT_ENTRY value
    push!(9u64);        // AT_ENTRY type
    push!(phdr_addr);   // AT_PHDR value
    push!(3u64);        // AT_PHDR type

    // envp
    push!(0u64);        // envp null terminator

    // argv
    push!(0u64);        // argv null terminator
    push!(prog_ptr);    // argv[0]

    // argc
    push!(1u64);

    // Switch to the new stack and jump — use jmp not call.
    // call would push a return address and shift argc down by 8 bytes.
    unsafe {
        std::arch::asm!(
            "mov rsp, {sp}",
            "jmp {entry}",
            sp = in(reg) sp,
            entry = in(reg) entry,
            options(noreturn),
        );
    }
}
