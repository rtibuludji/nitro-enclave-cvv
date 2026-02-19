#include <stdio.h>
#include <stdlib.h>

int main() {
    FILE *fp = fopen("/proc/sys/net/vsock/local_cid", "r");
    if (!fp) {
        fp = fopen("/sys/module/vsock/parameters/local_cid", "r");
    }
    
    if (!fp) {
        perror("Cannot find CID in sysfs/proc");
        return 1;
    }
    
    unsigned int cid;
    if (fscanf(fp, "%u", &cid) != 1) {
        fprintf(stderr, "Failed to read CID\n");
        fclose(fp);
        return 1;
    }
    
    printf("Local CID: %u\n", cid);
    fclose(fp);
    return 0;
}
