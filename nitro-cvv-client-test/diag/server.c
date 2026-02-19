#include <stdio.h>
#include <string.h>
#include <sys/socket.h>
#include <linux/vm_sockets.h>
#include <unistd.h>

int main() {
    int listen_fd = socket(AF_VSOCK, SOCK_STREAM, 0);
    if (listen_fd < 0) {
        perror("socket");
        return 1;
    }
    
    struct sockaddr_vm addr = {
        .svm_family = AF_VSOCK,
        .svm_port = 9999,
        .svm_cid = VMADDR_CID_ANY,
    };
    
    if (bind(listen_fd, (struct sockaddr*)&addr, sizeof(addr)) < 0) {
        perror("bind");
        return 1;
    }
    
    if (listen(listen_fd, 1) < 0) {
        perror("listen");
        return 1;
    }
    
    printf("VSOCK server listening on port 9999...\n");
    
    int conn_fd = accept(listen_fd, NULL, NULL);
    if (conn_fd < 0) {
        perror("accept");
        return 1;
    }
    
    char buf[1024];
    ssize_t n = read(conn_fd, buf, sizeof(buf)-1);
    if (n > 0) {
        buf[n] = '\0';
        printf("Received: %s\n", buf);
    }
    
    write(conn_fd, "Hello from server", 17);
    
    close(conn_fd);
    close(listen_fd);
    return 0;
}
