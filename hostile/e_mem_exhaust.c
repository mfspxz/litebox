// Hostile test (e): try exhausting memory.
// RLIMIT_AS should cap the child's address space; malloc/mmap must fail.

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main() {
    printf("attempting to exhaust memory...\n");
    fflush(stdout);
    size_t total = 0;
    for (int i = 0; i < 100; i++) {
        // Try to allocate 100MB at a time.
        void *p = malloc(100 * 1024 * 1024);
        if (!p) {
            printf("OK: malloc failed at %zu MB (RLIMIT_AS enforced)\n", total / (1024*1024));
            return 0;
        }
        // Touch the pages to force commit.
        memset(p, 0xAA, 100 * 1024 * 1024);
        total += 100 * 1024 * 1024;
        printf("allocated %zu MB...\n", total / (1024*1024));
        fflush(stdout);
    }
    printf("UNEXPECTED: allocated 10GB without failure\n");
    return 1;
}
