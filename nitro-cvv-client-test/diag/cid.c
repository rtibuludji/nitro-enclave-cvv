#include <stdio.h>
#include <fcntl.h>
#include <sys/socket.h>
#include <sys/ioctl.h>
#include <linux/vm_sockets.h>

int main() {
    int fd = open("/dev/vsock", O_RDONLY);
    if (fd < 0) {
        perror("open");
        return 1;
    }
    unsigned int cid;
    if (ioctl(fd, IOCTL_VM_SOCKETS_GET_LOCAL_CID, &cid) < 0) {
        perror("ioctl");
        return 1;
    }
    printf("Local CID: %u\n", cid);
    return 0;
}

