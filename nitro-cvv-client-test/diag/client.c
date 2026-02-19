#include <stdio.h>
#include <string.h>
#include <sys/socket.h>
#include <linux/vm_sockets.h>
#include <unistd.h>

#define VMADDR_CID_LOCAL 1  // Loopback CID

int main() {
    int fd = socket(AF_VSOCK, SOCK_STREAM, 0);
    if (fd < 0) {
        perror("socket");
        return 1;
    }
    
    struct sockaddr_vm addr = {
        .svm_family = AF_VSOCK,
        .svm_port = 9999,
        .svm_cid = VMADDR_CID_LOCAL,  // Connect to loopback
    };
    
    if (connect(fd, (struct sockaddr*)&addr, sizeof(addr)) < 0) {
        perror("connect");
        return 1;
    }
    
    printf("Connected via VSOCK!\n");
    
    write(fd, "Hello from client", 17);
    
    char buf[1024];
    ssize_t n = read(fd, buf, sizeof(buf)-1);
    if (n > 0) {
        buf[n] = '\0';
        printf("Received: %s\n", buf);
    }
    
    close(fd);
    return 0;
}
