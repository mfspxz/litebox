// Hostile test (c): normal socket() via libc (goes through LiteBox shim).
// The shim has no TUN device, so this should hang -> watchdog timeout.

#include <stdio.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <unistd.h>

int main() {
    printf("creating socket via shim...\n");
    fflush(stdout);
    int fd = socket(AF_INET, SOCK_STREAM, 0);
    if (fd < 0) {
        printf("socket failed (shim denied)\n");
        return 0;
    }
    printf("socket created, attempting connect (should hang)...\n");
    fflush(stdout);
    struct sockaddr_in addr = {
        .sin_family = AF_INET,
        .sin_port = htons(80),
        .sin_addr.s_addr = inet_addr("1.1.1.1"),
    };
    int rc = connect(fd, (struct sockaddr*)&addr, sizeof(addr));
    printf("connect returned %d (unexpected, should have hung)\n", rc);
    return 0;
}
