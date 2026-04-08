use std::convert::TryInto;

pub struct ElfHeader {
    pub entry: u64,
    pub phoff: u64,
    pub shoff: u64,
    pub phnum: u16,
    pub shnum: u16,
}

pub struct ProgramHeader {
    pub p_type: u32,
    pub flags: u32,
    pub p_offset: u64,
    pub vaddr: u64,
    pub paddr: u64,
    pub filesz: u64,
    pub memsz: u64,
    pub align: u64,
}

pub fn parse_elf_header(data: &[u8]) -> Result<ElfHeader, String> {
    if data.len() < 64 {
        return Err("Too small to be an ELF file".into());
    }

    if &data[0..4] != &[0x7F, b'E', b'L', b'F'] {
        return Err("Not an ELF file".into());
    }

    if data[4] != 2 {
        return Err("ELF file not 64-bit".into());
    }

    let entry = u64::from_le_bytes(data[24..32].try_into().unwrap());

    let phoff = u64::from_le_bytes(data[32..40].try_into().unwrap());
    let shoff = u64::from_le_bytes(data[40..48].try_into().unwrap());

    let phnum = u16::from_le_bytes(data[56..58].try_into().unwrap());
    let shnum = u16::from_le_bytes(data[60..62].try_into().unwrap());

    Ok(ElfHeader{entry, phoff, shoff, phnum, shnum})

}

pub fn parse_segments(data: &[u8], phoff: u64, phnum:u16,) -> Result<Vec<ProgramHeader>, String> {
    let mut headers = Vec::new();

    let ph_size = u16::from_le_bytes(
        data.get(54..56)
            .ok_or("Out of bounds")?
            .try_into()
            .map_err(|_| "Slice size incorrect")?
    );

    for i in 0..phnum {
        let offset = phoff as usize + (i as usize * ph_size as usize);

        let p_type = u32::from_le_bytes(data[offset as usize..(offset + 4) as usize].try_into().unwrap());
        let flags = u32::from_le_bytes(data[(offset + 4) as usize..(offset + 8) as usize].try_into().unwrap());

        let p_offset = u64::from_le_bytes(data[(offset + 8) as usize..(offset+16) as usize].try_into().unwrap());

        let vaddr = u64::from_le_bytes(data[(offset+16) as usize..(offset+24) as usize].try_into().unwrap());
        let paddr = u64::from_le_bytes(data[(offset+24) as usize..(offset+32) as usize].try_into().unwrap());

        let filesz = u64::from_le_bytes(data[(offset+32) as usize..(offset+40) as usize].try_into().unwrap());
        let memsz = u64::from_le_bytes(data[(offset+40) as usize..(offset+48) as usize].try_into().unwrap());
        let align = u64::from_le_bytes(data[(offset+48) as usize..(offset+56) as usize].try_into().unwrap());

        headers.push(ProgramHeader {
            p_type,
            flags,
            p_offset,
            vaddr,
            paddr,
            filesz,
            memsz,
            align,
        });
    }
    Ok(headers)
}

pub fn ph_type_to_str(t: u32) -> &'static str {
    match t {
        1 => "LOAD",
        2 => "DYNAMIC",
        3 => "INTERP",
        4 => "NOTE",
        5 => "SHLIB",
        6 => "PDHR",
        7 => "TLS",
        0 => "NULL",
        _ => "OTHER",
    }
}

pub fn flags_to_str(f: u32) -> String {
    let mut s = String::new();
    if f & 0x4 != 0 { s.push('R'); } else { s.push('-'); }
    if f & 0x2 != 0 { s.push('W'); } else { s.push('-'); }
    if f & 0x1 != 0 { s.push('X'); } else { s.push('-'); }
    s
}
