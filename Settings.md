
## Enable VSOCK in WSL Ubuntu (Manual)
 ```bash
sudo modprobe vsock
sudo modprobe vsock_diag
sudo modprobe vmw_vsock_virtio_transport_common
sudo modprobe vmw_vsock_virtio_transport
sudo modprobe vsock_loopback
sudo modprobe vhost_vsock
lsmod | grep vsock
```