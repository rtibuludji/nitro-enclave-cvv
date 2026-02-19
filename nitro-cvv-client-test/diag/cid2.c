#include <stdio.h>
#include <sys/socket.h>
#include <sys/ioctl.h>
#include <linux/vm_sockets.h>
#include <unistd.h>

int main() {
    int fd = socket(AF_VSOCK, SOCK_STREAM, 0);
    if (fd < 0) {
        perror("socket");
        return 1;
    }
    
    unsigned int cid;
    if (ioctl(fd, IOCTL_VM_SOCKETS_GET_LOCAL_CID, &cid) < 0) {
        perror("ioctl");
        close(fd);
        return 1;
    }
    
    printf("Local CID: %u\n", cid);
    close(fd);
    return 0;
}

