use nix::sys::socket::{
    self,
    sockopt::ReuseAddr,
    AddressFamily, 
    Backlog,
    SockFlag, 
    SockType, 
    SockaddrIn,
    SockaddrStorage, 
    VsockAddr
};
use nix::unistd::{self};
use std::io;
use std::net::{Ipv4Addr};
use std::os::fd::{AsRawFd, RawFd, FromRawFd, OwnedFd};
use std::marker::PhantomData;

use log::{debug, info, error};

pub trait SocketType {
    type Address;
    
    fn address_family() -> AddressFamily;
    fn socket_type() -> SockType;
    fn is_connection_oriented() -> bool;
}

pub struct Tcp;

impl SocketType for Tcp {
    type Address = SockaddrIn;
    
    fn address_family() -> AddressFamily {
        AddressFamily::Inet
    }
    
    fn socket_type() -> SockType {
        SockType::Stream
    }
    
    fn is_connection_oriented() -> bool {
        true
    }
}

pub struct Vsock;

impl SocketType for Vsock {
    type Address = VsockAddr;
    
    fn address_family() -> AddressFamily {
        AddressFamily::Vsock
    }
    
    fn socket_type() -> SockType {
        SockType::Stream
    }
    
    fn is_connection_oriented() -> bool {
        true
    }
}

pub struct Stream<P: SocketType> {
    fd: OwnedFd,
    _phantom: PhantomData<P>,
}

impl<P: SocketType> Stream<P> {
    fn from_raw_fd(rawfd: RawFd) -> Self {
        let fd: OwnedFd = unsafe { 
            OwnedFd::from_raw_fd(rawfd)
        };
        Stream {
            fd,
            _phantom: PhantomData,
        }
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        debug!("Reading from stream (fd={}, buf_size={})", self.fd.as_raw_fd(), buf.len());
        match unistd::read(&self.fd, buf) {
            Ok(n) => {
                debug!("Successfully read from stream (fd={}, bytes_read={})", self.fd.as_raw_fd(), n);
                Ok(n)
            },
            Err(e) => {
                error!("Failed to read from stream (fd={}): {:?}", self.fd.as_raw_fd(), e);
                Err(io::Error::new(io::ErrorKind::Other, e))
            }
        }
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        debug!("Writing to stream (fd={}, buf_size={})", self.fd.as_raw_fd(), buf.len());
        match unistd::write(&self.fd, buf) {
            Ok(n) => {
                debug!("Successfully wrote to stream (fd={}, bytes_written={})", self.fd.as_raw_fd(), n);
                Ok(n)
            },
            Err(e) => {
                error!("Failed to write to stream (fd={}): {:?}", self.fd.as_raw_fd(), e);
                Err(io::Error::new(io::ErrorKind::Other, e))
            }
        }
    }

    pub fn as_raw_fd(&self) -> RawFd {
        self.fd.as_raw_fd()
    }
}

impl<P: SocketType> Drop for Stream<P> {
    fn drop(&mut self) {
        debug!("Closing stream socket (fd={})", self.fd.as_raw_fd());
    }
}

pub struct Listener<P: SocketType> {
    fd:       OwnedFd,
    _phantom: PhantomData<P>,
}

impl<P: SocketType> Listener<P> {
    pub fn as_raw_fd(&self) -> RawFd {
        self.fd.as_raw_fd()
    }
}

impl<P: SocketType> Drop for Listener<P> {
    fn drop(&mut self) {
        debug!("Closing listener socket (fd={})", self.fd.as_raw_fd());
    }
}

impl Listener<Tcp> {
    pub fn bind_tcp(addr: Ipv4Addr, port: u16) -> io::Result<Self> {
        info!("Creating TCP socket");
        let fd = socket::socket(
            Tcp::address_family(),
            Tcp::socket_type(),
            SockFlag::empty(),
            None,
        )
        .map_err(|e| {
            error!("Failed to create TCP socket: {:?}", e);
            io::Error::new(io::ErrorKind::Other, e)
        })?;

        debug!("Setting SO_REUSEADDR socket option");
        socket::setsockopt(&fd, ReuseAddr, &true)
            .map_err(|e| {
                error!("Failed to set SO_REUSEADDR: {:?}", e);
                io::Error::new(io::ErrorKind::Other, e)
            })?;

        let sockaddr = SockaddrIn::new(
            addr.octets()[0],
            addr.octets()[1],
            addr.octets()[2],
            addr.octets()[3],
            port,
        );

        let backlog = Backlog::new(128)
            .map_err(|e| {
                error!("Failed to create backlog: {:?}", e);
                io::Error::new(io::ErrorKind::InvalidInput, e)
            })?;

        debug!("Binding to TCP address");
        socket::bind(fd.as_raw_fd(), &sockaddr)
            .map_err(|e| {
                error!("Failed to bind TCP socket: {:?}", e);
                io::Error::new(io::ErrorKind::Other, e)
            })?;

        debug!("Starting to listen on TCP socket");
        socket::listen(&fd, backlog)
            .map_err(|e| {
                error!("Failed to listen on TCP socket: {:?}", e);
                io::Error::new(io::ErrorKind::Other, e)
            })?;

        info!("TCP listener successfully bound to {}:{}", addr, port);
        Ok(Listener{fd, _phantom: PhantomData})
    }

    pub fn accept(&self) -> io::Result<(Stream<Tcp>, SockaddrStorage)> {
        debug!("Waiting to accept TCP connection");
        let conn_fd: RawFd = socket::accept(self.fd.as_raw_fd())
            .map_err(|e| {
                error!("Failed to accept TCP connection: {:?}", e);
                io::Error::new(io::ErrorKind::Other, e)
            })?;

        let addr = socket::getpeername(conn_fd)
            .map_err(|e| {
                error!("Failed to get peer name (fd={}): {:?}", conn_fd, e);
                io::Error::new(io::ErrorKind::Other, e)
            })?;

        info!("Accepted new TCP connection (fd={})", conn_fd);
        Ok((Stream::from_raw_fd(conn_fd), addr))
    }
}

impl Listener<Vsock> {
    pub fn bind_vsock(cid: u32, port: u32) -> io::Result<Self> {
        info!("Creating Vsock socket");
        let fd = socket::socket(
            Vsock::address_family(),
            Vsock::socket_type(),
            SockFlag::empty(),
            None,
        )
        .map_err(|e| {
            error!("Failed to create Vsock socket: {:?}", e);
            io::Error::new(io::ErrorKind::Other, e)
        })?;

        let sockaddr = VsockAddr::new(cid, port);

        let backlog = Backlog::new(128)
            .map_err(|e| {
                error!("Failed to create backlog: {:?}", e);
                io::Error::new(io::ErrorKind::InvalidInput, e)
            })?;

        debug!("Binding to Vsock address");
        socket::bind(fd.as_raw_fd(), &sockaddr)
            .map_err(|e| {
                error!("Failed to bind Vsock socket: {:?}", e);
                io::Error::new(io::ErrorKind::Other, e)
            })?;

        debug!("Starting to listen on Vsock socket");
        socket::listen(&fd, backlog)
            .map_err(|e| {
                error!("Failed to listen on Vsock socket: {:?}", e);
                io::Error::new(io::ErrorKind::Other, e)
            })?;

        info!("Vsock listener successfully bound to {}:{}", cid, port);
        Ok(Listener{fd, _phantom: PhantomData})
    }

    pub fn accept(&self) -> io::Result<(Stream<Vsock>, SockaddrStorage)> {
        debug!("Waiting to accept Vsock connection");
        let conn_fd = socket::accept(self.as_raw_fd())
            .map_err(|e| {
                error!("Failed to accept Vsock connection: {:?}", e);
                io::Error::new(io::ErrorKind::Other, e)
            })?;

        let addr = socket::getpeername(conn_fd.as_raw_fd())
            .map_err(|e| {
                error!("Failed to get peer name (fd={}): {:?}", conn_fd.as_raw_fd(), e);
                io::Error::new(io::ErrorKind::Other, e)
            })?;

        info!("Accepted new Vsock connection (fd={})", conn_fd.as_raw_fd());
        Ok((Stream::from_raw_fd(conn_fd), addr))
    }
}

pub struct Client<P: SocketType> {
    _phantom: PhantomData<P>,
}

impl Client<Tcp> {
    pub fn connect_tcp(addr: Ipv4Addr, port: u16) -> io::Result<Stream<Tcp>> {
        info!("Creating TCP client socket");
        let fd = socket::socket(
            Tcp::address_family(),
            Tcp::socket_type(),
            SockFlag::empty(),
            None,
        )
        .map_err(|e| {
            error!("Failed to create TCP client socket: {:?}", e);
            io::Error::new(io::ErrorKind::Other, e)
        })?;

        let sockaddr = SockaddrIn::new(
            addr.octets()[0],
            addr.octets()[1],
            addr.octets()[2],
            addr.octets()[3],
            port,
        );

        debug!("Connecting to TCP server");
        socket::connect(fd.as_raw_fd(), &sockaddr)
            .map_err(|e| {
                error!("Failed to connect to TCP server: {:?}", e);
                io::Error::new(io::ErrorKind::Other, e)
            })?;

        info!("Successfully connected to {}:{}", addr, port);
        Ok(Stream{fd, _phantom: PhantomData})
    }
}

impl Client<Vsock> {
    pub fn connect_vsock(cid: u32, port: u32) -> io::Result<Stream<Vsock>> {
        info!("Creating Vsock client socket");
        let fd = socket::socket(
            Vsock::address_family(),
            Vsock::socket_type(),
            SockFlag::empty(),
            None,
        )
        .map_err(|e| {
            error!("Failed to create Vsock client socket: {:?}", e);
            io::Error::new(io::ErrorKind::Other, e)
        })?;

        let sockaddr = VsockAddr::new(cid, port);

        debug!("Connecting to Vsock server");
        socket::connect(fd.as_raw_fd(), &sockaddr)
            .map_err(|e| {
                error!("Failed to connect to Vsock server: {:?}", e);
                io::Error::new(io::ErrorKind::Other, e)
            })?;

        info!("Successfully connected to {}:{}", cid, port);
        Ok(Stream{fd, _phantom: PhantomData})
    }
}
