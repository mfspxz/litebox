// Hostile test (d): try writing outside the allowlist.
// - /etc/evil: should fail (not in virtual FS, not in mount ns)
// - /tmp/evil: should succeed (writable scratch)
// - /bin/evil: should fail (read-only)

#include <stdio.h>
#include <fcntl.h>
#include <unistd.h>
#include <string.h>

void try_write(const char *path) {
    int fd = open(path, O_WRONLY | O_CREAT, 0644);
    if (fd < 0) {
        printf("OK: write to %s denied\n", path);
        return;
    }
    const char *data = "evil";
    ssize_t n = write(fd, data, strlen(data));
    close(fd);
    if (n > 0) {
        printf("WRITE SUCCEEDED: %s (%zd bytes)\n", path, n);
    } else {
        printf("OK: write to %s failed on write\n", path);
    }
}

int main() {
    try_write("/etc/evil");
    try_write("/tmp/evil");
    try_write("/bin/evil");
    return 0;
}
