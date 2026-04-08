use libc::{mmap, MAP_FAILED, MAP_ANONYMOUS, MAP_PRIVATE, MAP_FIXED, PROT_EXEC, PROT_READ, PROT_WRITE};
use std::ptr;
use crate::elf::ProgramHeader;

pub unsafe fn load_segments(data: &[u8], phdrs: &[ProgramHeader]) {
    for seg in phdrs {
        if seg.p_type != 1 || seg.memsz == 0 {
            continue;
        }

        // Page-align the mapping address and size.
        let page_size: u64 = 4096;
        let align_down = |x: u64| x & !(page_size - 1);
        let align_up   = |x: u64| (x + page_size - 1) & !(page_size - 1);

        let map_start = align_down(seg.vaddr);
        let map_end   = align_up(seg.vaddr + seg.memsz);
        let map_size  = (map_end - map_start) as usize;

        unsafe {
            // MAP_FIXED: must land exactly at map_start — required for non-PIE binaries
            // whose code contains hardcoded absolute addresses.
            let addr = mmap(
                map_start as *mut _,
                map_size,
                PROT_READ | PROT_WRITE | PROT_EXEC,
                MAP_PRIVATE | MAP_ANONYMOUS | MAP_FIXED,
                -1,
                0,
            );
            if addr == MAP_FAILED {
                panic!("mmap failed for segment at 0x{:x}", seg.vaddr);
            }

            // Copy file contents into the mapped region.
            ptr::copy_nonoverlapping(
                data.as_ptr().add(seg.p_offset as usize),
                seg.vaddr as *mut u8,
                seg.filesz as usize,
            );

            // Zero the .bss region (memsz > filesz).
            if seg.memsz > seg.filesz {
                ptr::write_bytes(
                    (seg.vaddr as *mut u8).add(seg.filesz as usize),
                    0,
                    (seg.memsz - seg.filesz) as usize,
                );
            }
        }
    }
}
