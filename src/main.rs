mod elf;
mod loader;
mod process;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: paasa <command> <file>");
        return;
    }
    let command = &args[1];
    let file = &args[2];

    let data = match fs::read(file) {
        Ok(d) => d,
        Err(e) => { eprintln!("failed to read file: {}", e); return; }
    };

    let header = match elf::parse_elf_header(&data) {
        Ok(h) => h,
        Err(e) => { eprintln!("Error parsing ELF: {}", e); return; }
    };

    let phdrs = match elf::parse_segments(&data, header.phoff, header.phnum) {
        Ok(p) => p,
        Err(e) => { eprintln!("Error parsing program headers: {}", e); return; }
    };

    if command == "inspect" {
        println!("ELF Header:");
        println!("  Entry Point:          0x{:x}", header.entry);
        println!("  Program Header offset: 0x{:x}", header.phoff);
        println!("  Section Header offset: 0x{:x}", header.shoff);
        println!("  Program Header count:  {}", header.phnum);
        println!("  Section Header count:  {}", header.shnum);
        println!("\nProgram Headers:");
        for ph in &phdrs {
            println!(
                "  {:<8} vaddr=0x{:016x} {} filesz=0x{:x} memsz=0x{:x}",
                elf::ph_type_to_str(ph.p_type),
                ph.vaddr,
                elf::flags_to_str(ph.flags),
                ph.filesz,
                ph.memsz
            );
        }
    } else if command == "load" {
        // Compute phdr_addr: where the program headers will live in memory after loading.
        // For a non-PIE static binary this is simply phoff bytes past the first LOAD vaddr.
        let load_base = phdrs.iter()
            .find(|p| p.p_type == 1)
            .map(|p| p.vaddr)
            .unwrap_or(0);
        let phdr_addr = load_base + header.phoff;

        unsafe {
            loader::load_segments(&data, &phdrs);
            process::start(header.entry, phdr_addr);
        }
    } else {
        eprintln!("Unknown command: {}", command);
    }
}
