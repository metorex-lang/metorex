// The descriptor behind a socket, and the descriptors handed over one.

use super::*;

impl VirtualMachine {
    pub(crate) fn socket_fileno(&mut self, handle: u64) -> Result<Object, MetorexError> {
        if let Some(found) = self.socket_descriptor(handle) {
            return Ok(Object::Int(i64::from(found)));
        }
        use std::os::unix::io::AsRawFd as _;
        let found = self
            .open_sockets
            .listeners
            .get(&handle)
            .map(|held| held.as_raw_fd())
            .or_else(|| {
                self.open_sockets
                    .streams
                    .get(&handle)
                    .map(|h| h.as_raw_fd())
            })
            .or_else(|| {
                self.open_sockets
                    .unix_listeners
                    .get(&handle)
                    .map(|h| h.as_raw_fd())
            })
            .or_else(|| {
                self.open_sockets
                    .unix_streams
                    .get(&handle)
                    .map(|h| h.as_raw_fd())
            })
            .or_else(|| {
                self.open_sockets
                    .datagrams
                    .get(&handle)
                    .map(|h| h.as_raw_fd())
            });
        Ok(match found {
            Some(held) => Object::Int(i64::from(held)),
            None => Object::Nil,
        })
    }

    /// The socket a descriptor names, answered as the handle metorex
    /// holds it under, which is what `for_fd` builds another socket
    /// around. A socket the program did not open is taken up first.
    pub(crate) fn socket_handle_of_descriptor(
        &mut self,
        port: i64,
    ) -> Result<Object, MetorexError> {
        use std::os::unix::io::AsRawFd as _;
        let wanted = port as i32;
        let found = self
            .open_sockets
            .listeners
            .iter()
            .map(|(held, socket)| (*held, socket.as_raw_fd()))
            .chain(
                self.open_sockets
                    .streams
                    .iter()
                    .map(|(held, socket)| (*held, socket.as_raw_fd())),
            )
            .chain(
                self.open_sockets
                    .unix_listeners
                    .iter()
                    .map(|(held, socket)| (*held, socket.as_raw_fd())),
            )
            .chain(
                self.open_sockets
                    .unix_streams
                    .iter()
                    .map(|(held, socket)| (*held, socket.as_raw_fd())),
            )
            .chain(
                self.open_sockets
                    .datagrams
                    .iter()
                    .map(|(held, socket)| (*held, socket.as_raw_fd())),
            )
            .chain(
                self.open_sockets
                    .unix_datagrams
                    .iter()
                    .map(|(held, socket)| (*held, socket.as_raw_fd())),
            )
            .find(|(_, number)| *number == wanted)
            .map(|(held, _)| held);
        Ok(match found.or_else(|| self.adopt_inherited_socket(wanted)) {
            Some(held) => Object::Int(held as i64),
            None => Object::Nil,
        })
    }

    /// Hold a socket this program did not open, such as one a parent
    /// process handed on, under a handle of its own, as the kind of socket
    /// the operating system says it is. A descriptor that is no socket is
    /// left alone.
    fn adopt_inherited_socket(&mut self, descriptor: i32) -> Option<u64> {
        use std::os::unix::io::FromRawFd as _;
        let option = |name: libc::c_int| {
            let mut value: libc::c_int = 0;
            let mut length = std::mem::size_of::<libc::c_int>() as libc::socklen_t;
            // SAFETY: `value` holds an int, and `length` says so.
            let answered = unsafe {
                libc::getsockopt(
                    descriptor,
                    libc::SOL_SOCKET,
                    name,
                    &mut value as *mut libc::c_int as *mut libc::c_void,
                    &mut length,
                )
            };
            (answered == 0).then_some(value)
        };
        let kind = option(libc::SO_TYPE)?;
        let address_of = |ask: unsafe extern "C" fn(
            libc::c_int,
            *mut libc::sockaddr,
            *mut libc::socklen_t,
        ) -> libc::c_int| {
            // SAFETY: an all-zero sockaddr_storage is a valid value, and
            // `length` says how much of it there is.
            let mut address: libc::sockaddr_storage = unsafe { std::mem::zeroed() };
            let mut length = std::mem::size_of::<libc::sockaddr_storage>() as libc::socklen_t;
            // SAFETY: as above.
            let answered = unsafe {
                ask(
                    descriptor,
                    &mut address as *mut libc::sockaddr_storage as *mut libc::sockaddr,
                    &mut length,
                )
            };
            (answered == 0).then_some(address)
        };
        let address = address_of(libc::getsockname)?;
        // macOS does not answer SO_ACCEPTCONN, so a stream socket with no
        // other end is taken to be listening.
        let listening = match option(libc::SO_ACCEPTCONN) {
            Some(value) => value != 0,
            None => address_of(libc::getpeername).is_none(),
        };
        let unix = libc::c_int::from(address.ss_family) == libc::AF_UNIX;
        let handle = self.open_sockets.next;
        // SAFETY: the descriptor is an open socket of the kind and family
        // asked above, which the object built here owns from now on, as
        // the Ruby object built around it does.
        unsafe {
            match (kind, listening, unix) {
                (libc::SOCK_STREAM, true, false) => {
                    let held = std::net::TcpListener::from_raw_fd(descriptor);
                    let _ = held.set_nonblocking(true);
                    self.open_sockets.listeners.insert(handle, held);
                }
                (libc::SOCK_STREAM, false, false) => {
                    let held = std::net::TcpStream::from_raw_fd(descriptor);
                    let _ = held.set_read_timeout(Some(POLL_LIMIT));
                    let _ = held.set_nonblocking(true);
                    self.open_sockets.streams.insert(handle, held);
                }
                (libc::SOCK_STREAM, true, true) => {
                    let held = std::os::unix::net::UnixListener::from_raw_fd(descriptor);
                    let _ = held.set_nonblocking(true);
                    self.open_sockets.unix_listeners.insert(handle, held);
                }
                (libc::SOCK_STREAM, false, true) => {
                    let held = std::os::unix::net::UnixStream::from_raw_fd(descriptor);
                    let _ = held.set_read_timeout(Some(POLL_LIMIT));
                    let _ = held.set_nonblocking(true);
                    self.open_sockets.unix_streams.insert(handle, held);
                }
                (libc::SOCK_DGRAM, _, false) => {
                    let held = std::net::UdpSocket::from_raw_fd(descriptor);
                    let _ = held.set_read_timeout(Some(POLL_LIMIT));
                    let _ = held.set_nonblocking(true);
                    self.open_sockets.datagrams.insert(handle, held);
                }
                (libc::SOCK_DGRAM, _, true) => {
                    let held = std::os::unix::net::UnixDatagram::from_raw_fd(descriptor);
                    let _ = held.set_read_timeout(Some(POLL_LIMIT));
                    let _ = held.set_nonblocking(true);
                    self.open_sockets.unix_datagrams.insert(handle, held);
                }
                _ => return None,
            }
        }
        self.open_sockets.next += 1;
        Some(handle)
    }

    /// The port a named service is reached on, and the name a port is
    /// known by, as the operating system's own table has them.
    /// The number this platform names a socket setting, a family, or a
    /// message flag by. The names differ between platforms, so a
    /// program that writes one of them by hand would be writing the
    /// number another platform uses.
    pub(crate) fn socket_constant(&mut self, text: String) -> Result<Object, MetorexError> {
        Ok(socket_constant(&text)
            .map(Object::Int)
            .unwrap_or(Object::Nil))
    }

    /// Ask the device behind the socket to do something, with a
    /// buffer it reads from and writes back into.
    pub(crate) fn socket_ioctl(
        &mut self,
        handle: u64,
        text: String,
        port: i64,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let Some(descriptor) = self.socket_descriptor(handle) else {
            return Err(refused(position, "ioctl on a closed socket".to_string()));
        };
        let mut buffer = crate::vm::native_methods::pack_format::string_to_bytes(&text);
        let request = port as libc::c_ulong;
        // SAFETY: the descriptor is one this program holds open, and
        // the buffer outlives the call.
        let answered = if buffer.is_empty() {
            unsafe { libc::ioctl(descriptor, request, 0) }
        } else {
            unsafe { libc::ioctl(descriptor, request, buffer.as_mut_ptr()) }
        };
        if answered < 0 {
            let problem = std::io::Error::last_os_error();
            return Err(crate::vm::errors::simple_exception(
                errno_class(&problem),
                &format!("ioctl: {problem}"),
                position,
            ));
        }
        Ok(Object::array(vec![
            Object::Int(i64::from(answered)),
            crate::vm::native_methods::pack_format::bytes_to_string(&buffer),
        ]))
    }

    /// Whether the socket answers straight away rather than waiting,
    /// as the descriptor itself is set, and setting it.
    pub(crate) fn socket_nonblock(&mut self, handle: u64) -> Result<Object, MetorexError> {
        let Some(descriptor) = self.socket_descriptor(handle) else {
            return Ok(Object::Nil);
        };
        // SAFETY: the descriptor is one this program holds open.
        let flags = unsafe { libc::fcntl(descriptor, libc::F_GETFL) };
        Ok(Object::Bool(flags >= 0 && flags & libc::O_NONBLOCK != 0))
    }

    pub(crate) fn socket_set_nonblock(
        &mut self,
        handle: u64,
        port: i64,
    ) -> Result<Object, MetorexError> {
        let Some(descriptor) = self.socket_descriptor(handle) else {
            return Ok(Object::Nil);
        };
        // SAFETY: the descriptor is one this program holds open.
        unsafe {
            let flags = libc::fcntl(descriptor, libc::F_GETFL);
            if flags >= 0 {
                let wanted = if port != 0 {
                    flags | libc::O_NONBLOCK
                } else {
                    flags & !libc::O_NONBLOCK
                };
                libc::fcntl(descriptor, libc::F_SETFL, wanted);
            }
        }
        Ok(Object::Bool(port != 0))
    }

    /// Hand a descriptor to the other end of a socket named by a
    /// path, and take one that was handed over. `port` carries the
    /// descriptor going out.
    pub(crate) fn socket_send_descriptor(
        &mut self,
        handle: u64,
        port: i64,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let Some(descriptor) = self.socket_descriptor(handle) else {
            return Err(refused(position, "send on a closed socket".to_string()));
        };
        let sent = send_descriptor(descriptor, port as i32);
        if sent < 0 {
            let problem = std::io::Error::last_os_error();
            return Err(crate::vm::errors::simple_exception(
                errno_class(&problem),
                &format!("sendmsg: {problem}"),
                position,
            ));
        }
        Ok(Object::Int(sent as i64))
    }

    pub(crate) fn socket_receive_descriptor(
        &mut self,
        handle: u64,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let Some(descriptor) = self.socket_descriptor(handle) else {
            return Err(refused(position, "receive on a closed socket".to_string()));
        };
        let taken = receive_descriptor(descriptor);
        if taken < 0 {
            let problem = std::io::Error::last_os_error();
            return Err(crate::vm::errors::simple_exception(
                errno_class(&problem),
                &format!("recvmsg: {problem}"),
                position,
            ));
        }
        Ok(Object::Int(taken as i64))
    }

    /// Which kind of socket a handle names, so one taken up from a
    /// file descriptor reports the family and the kind it really is.
    pub(crate) fn socket_kind_of_handle(&mut self, handle: u64) -> Result<Object, MetorexError> {
        let named = if self.open_sockets.listeners.contains_key(&handle)
            || self.open_sockets.streams.contains_key(&handle)
        {
            "inet_stream"
        } else if self.open_sockets.unix_listeners.contains_key(&handle)
            || self.open_sockets.unix_streams.contains_key(&handle)
        {
            "unix_stream"
        } else if self.open_sockets.datagrams.contains_key(&handle) {
            "inet_datagram"
        } else if self.open_sockets.unix_datagrams.contains_key(&handle) {
            "unix_datagram"
        } else {
            "unknown"
        };
        Ok(Object::string(named))
    }
}
