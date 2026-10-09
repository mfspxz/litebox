// Hostile test (a): try reading /proc/self/mem.
// First via normal open (goes through LiteBox shim -> ENOENT, no /proc).
// Then via raw syscall bypass (runtime-generated) -> outer mount ns has no /proc.

#include <stdio.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/mman.h>
#include <string.h>

int main() {
    // Attempt 1: normal open (shim-emulated).
    int fd = open("/proc/self/mem", O_RDONLY);
    if (fd >= 0) {
        printf("UNEXPECTED: normal open succeeded\n");
        close(fd);
    } else {
        printf("OK: normal open failed (shim has no /proc)\n");
    }

    // Attempt 2: raw syscall bypass. Generate the syscall instruction at
    // runtime so the static rewriter cannot see it.
    // Bytes: mov $2, %rax (open=2); mov $path, %rdi; mov $0, %rsi; syscall; ret
    // We use a simpler approach: call the raw syscall via inline bytes.
    unsigned char code[] = {
        0x48, 0xc7, 0xc0, 0x02, 0x00, 0x00, 0x00,  // mov $2, %rax (sys_open)
        0x48, 0xbf, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mov $path, %rdi (patched)
        0x48, 0xc7, 0xc6, 0x00, 0x00, 0x00, 0x00,  // mov $0, %rsi (O_RDONLY)
        0x0f, 0x05,                                // syscall
        0xc3                                       // ret
    };
    // Patch the path address (RDI).
    const char *path = "/proc/self/mem";
    memcpy(&code[9], &path, 8);

    void *mem = mmap(NULL, 4096, PROT_READ | PROT_WRITE | PROT_EXEC,
                     MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if (mem == MAP_FAILED) {
        printf("mmap failed\n");
        return 1;
    }
    memcpy(mem, code, sizeof(code));
    long (*fn)() = (long (*)())mem;
    long ret = fn();
    if (ret >= 0) {
        printf("UNEXPECTED: raw open succeeded, fd=%ld\n", ret);
    } else {
        printf("OK: raw open failed (outer mount ns has no /proc), err=%ld\n", -ret);
    }
    return 0;
}
