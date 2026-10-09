// Hostile test (b): raw syscall bypass via `int $0x80`.
// The static rewriter targets `syscall` (0F 05); `int $0x80` (CD 80) is a
// different instruction that invokes the 32-bit compat syscall handler.
// If the rewriter misses it, the host kernel sees the syscall and the outer
// seccomp-BPF deny-by-default must kill the process with SIGSYS.

#include <stdio.h>

int main() {
    printf("attempting raw socket via int $0x80...\n");
    fflush(stdout);

    long ret;
    // 32-bit socketcall: eax=102 (socketcall), ebx=1 (SYS_SOCKET),
    // ecx=ptr to [AF_INET, SOCK_STREAM, 0]
    long args[3] = {2, 1, 0};  // AF_INET=2, SOCK_STREAM=1, protocol=0
    __asm__ volatile (
        "int $0x80"
        : "=a" (ret)
        : "a" (102), "b" (1), "c" (args)
        : "memory"
    );
    // If we get here, seccomp did NOT kill us.
    printf("UNEXPECTED: int $0x80 socket returned %ld (bypass not contained!)\n", ret);
    return 0;
}
