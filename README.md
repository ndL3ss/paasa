# paasa

A toy ELF loader written in Rust. Built to understand what happens before `main` gets called — how segments get mapped into memory, how the stack is set up, and what the kernel normally does during `execve`.

Not intended for real use.

## What it does

- Parses ELF headers and program headers from raw bytes
- `inspect` — prints a summary of segments, section counts, and the entry point
- `load` — maps `PT_LOAD` segments into virtual memory using `mmap`, sets up a basic stack (argc, argv, envp, auxv), and jumps to the entry point

## Usage

```bash
cargo run -- inspect <elf_binary>
cargo run -- load <elf_binary>
```

## Test binary

A minimal no-libc binary to test with:

```c
// compile: gcc -o hello hello.c -static -nostdlib -e main
void main(void) {
    const char msg[] = "hello\n";
    __asm__ volatile (
        "syscall"
        : : "a"(1), "D"(1), "S"(msg), "d"(sizeof(msg) - 1)
        : "rcx", "r11", "memory"
    );
    __asm__ volatile (
        "syscall"
        : : "a"(60), "D"(0) :
    );
}
```
