use std::{cmp, fmt, io, mem, net};

use gleipnir_interface::Proto;
use pnet_macros_support::packet::{Packet, PacketSize};
use pnetlink::{
    packet::netlink::{NetlinkMsgFlags, NetlinkReader, NetlinkRequestBuilder},
    socket::{NetlinkProtocol, NetlinkSocket},
};

pub struct SockDiag {
    socket: NetlinkSocket,
}

impl SockDiag {
    pub fn new() -> io::Result<SockDiag> {
        let socket = NetlinkSocket::bind(NetlinkProtocol::Inet_diag, 0)?;
        Ok(SockDiag { socket })
    }

    pub fn query<'a>(
        &'a mut self,
        protocol: Proto,
        local_address: net::SocketAddr,
        remote_address: net::SocketAddr,
    ) -> Result<InetDiagMsg, io::Error> {
        const SOCK_DIAG_BY_FAMILY: u16 = 20;
        const INET_DIAG_NOCOOKIE: u32 = !0;

        assert_eq!(local_address.is_ipv4(), remote_address.is_ipv4());

        let req = InetDiagReqV2 {
            sdiag_family: if local_address.is_ipv4() {
                libc::AF_INET
            } else {
                libc::AF_INET6
            } as u8,
            sdiag_protocol: protocol as u8,
            idiag_ext: 0,
            pad: 0,
            idiag_states: !0, // any state
            id: InetDiagSockId {
                idiag_sport: local_address.port().into(),
                idiag_dport: remote_address.port().into(),
                idiag_src: local_address.ip().into(),
                idiag_dst: remote_address.ip().into(),
                idiag_if: 0,
                idiag_cookie: [INET_DIAG_NOCOOKIE; 2],
            },
        };

        let mut flags = NetlinkMsgFlags::NLM_F_REQUEST;
        if protocol != Proto::Tcp {
            // TODO: do we really need this for UDP?
            flags.insert(NetlinkMsgFlags::NLM_F_MATCH);
        }

        let req = NetlinkRequestBuilder::new(SOCK_DIAG_BY_FAMILY, flags)
            .append(req)
            .build();
        self.socket.send(req.packet())?;

        let mut r = None;
        let responses = NetlinkReader::new(&mut self.socket);
        for msg in responses {
            let diag_msg = msg.payload() as *const _ as *const InetDiagMsg;
            let diag_msg = unsafe { &(*diag_msg) };
            // filter for UDP
            if diag_msg.id.idiag_src == local_address.ip()
                && diag_msg.id.idiag_sport == local_address.port()
                && diag_msg.id.idiag_dst == remote_address.ip()
                && diag_msg.id.idiag_dport == remote_address.port()
                && diag_msg.idiag_inode != 0
            {
                r = Some(*diag_msg);
            }
        }

        r.ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))

        // let n = self.socket.recv(&mut self.buf)?;
        // if let Some(msg) = NetlinkIterable::new(&self.buf[..n]).next() {
        //     if msg.get_kind() == NLMSG_ERROR || msg.get_kind() == NLMSG_DONE {
        //         return Err(io::Error::from(io::ErrorKind::NotFound));
        //     }
        //     let diag_msg = msg.payload() as *const _ as *const InetDiagMsg;
        //     let diag_msg = unsafe { &(*diag_msg) };
        //     // make sure socket is empty
        //     match self.socket.recv(&mut [0u8; 64]) {
        //         Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => (),
        //         Err(e) => return Err(e),
        //         Ok(_) => {
        //             return Err(io::Error::new(
        //                 io::ErrorKind::InvalidData,
        //                 "SockDiag::find_one got more than one response",
        //             ));
        //         }
        //     }
        //     Ok(diag_msg)
        // } else {
        //     Err(io::Error::from(io::ErrorKind::NotFound))
        // }
    }
}

#[repr(C)]
#[derive(Debug)]
struct InetDiagReqV2 {
    sdiag_family: u8,
    sdiag_protocol: u8,
    idiag_ext: u8,
    pad: u8,
    idiag_states: u32,
    id: InetDiagSockId,
}

impl Packet for InetDiagReqV2 {
    fn packet(&self) -> &[u8] {
        let p: &[u8; mem::size_of::<Self>()] = unsafe { mem::transmute(self) };
        p
    }
    fn payload(&self) -> &[u8] {
        &[]
    }
}

impl PacketSize for InetDiagReqV2 {
    fn packet_size(&self) -> usize {
        mem::size_of::<Self>()
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct InetDiagMsg {
    pub idiag_family: u8,
    pub idiag_state: u8,
    pub idiag_timer: u8,
    pub idiag_retrans: u8,
    pub id: InetDiagSockId,
    pub idiag_expires: u32,
    pub idiag_rqueue: u32,
    pub idiag_wqueue: u32,
    pub idiag_uid: u32,
    pub idiag_inode: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct InetDiagSockId {
    pub idiag_sport: Port,
    pub idiag_dport: Port,
    pub idiag_src: Ipv4or6,
    pub idiag_dst: Ipv4or6,
    pub idiag_if: u32,
    pub idiag_cookie: [u32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Port([u8; 2]); // u16be

impl From<u16> for Port {
    fn from(port: u16) -> Self {
        Port([(port >> 8) as u8, port as u8])
    }
}

impl From<Port> for u16 {
    fn from(port: Port) -> Self {
        ((port.0[0] as u16) << 8) | (port.0[1] as u16)
    }
}

impl cmp::PartialEq<u16> for Port {
    fn eq(&self, other: &u16) -> bool {
        u16::from(*self) == *other
    }
}

impl fmt::Debug for Port {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        u16::from(*self).fmt(f)
    }
}

// [u32be; 4]
#[repr(C)]
#[derive(Clone, Copy)]
pub union Ipv4or6 {
    v4: [u8; 4],
    v6: [u8; 16],
}

// TODO: zero copy
impl From<net::Ipv4Addr> for Ipv4or6 {
    fn from(addr: net::Ipv4Addr) -> Self {
        Ipv4or6 { v4: addr.octets() }
    }
}

impl From<Ipv4or6> for net::Ipv4Addr {
    fn from(addr: Ipv4or6) -> Self {
        unsafe { addr.v4.into() }
    }
}

// TODO: zero copy
impl From<net::Ipv6Addr> for Ipv4or6 {
    fn from(addr: net::Ipv6Addr) -> Self {
        #[inline]
        fn l(v: u16) -> u8 {
            (v >> 8) as u8
        }
        #[inline]
        fn r(v: u16) -> u8 {
            v as u8
        }
        let v6: [u16; 8] = addr.segments();
        let v6: [u8; 16] = [
            l(v6[0]),
            r(v6[0]),
            l(v6[1]),
            r(v6[1]),
            l(v6[2]),
            r(v6[2]),
            l(v6[3]),
            r(v6[3]),
            l(v6[4]),
            r(v6[4]),
            l(v6[5]),
            r(v6[5]),
            l(v6[6]),
            r(v6[6]),
            l(v6[7]),
            r(v6[7]),
        ];
        Ipv4or6 { v6 }
    }
}

impl From<Ipv4or6> for net::Ipv6Addr {
    fn from(addr: Ipv4or6) -> Self {
        unsafe { addr.v6.into() }
    }
}

impl From<net::IpAddr> for Ipv4or6 {
    fn from(addr: net::IpAddr) -> Self {
        use std::net::IpAddr::*;
        match addr {
            V4(addr) => addr.into(),
            V6(addr) => addr.into(),
        }
    }
}

impl cmp::PartialEq<net::IpAddr> for Ipv4or6 {
    fn eq(&self, other: &net::IpAddr) -> bool {
        match other {
            net::IpAddr::V4(other) => self == other,
            net::IpAddr::V6(other) => self == other,
        }
    }
}

impl cmp::PartialEq<net::Ipv4Addr> for Ipv4or6 {
    fn eq(&self, other: &net::Ipv4Addr) -> bool {
        net::Ipv4Addr::from(*self) == *other
    }
}

impl cmp::PartialEq<net::Ipv6Addr> for Ipv4or6 {
    fn eq(&self, other: &net::Ipv6Addr) -> bool {
        net::Ipv6Addr::from(*self) == *other
    }
}

impl fmt::Debug for Ipv4or6 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Ipv4or6")
            .field("v4", &net::Ipv4Addr::from(*self))
            .field("v6", &net::Ipv6Addr::from(*self))
            .finish()
    }
}

#[test]
fn ipv4or6_convert_stdipaddr() {
    let v4: net::Ipv4Addr = "127.0.0.1".parse().unwrap();
    let ipv4or6: Ipv4or6 = v4.into();
    assert_eq!(net::Ipv4Addr::from(ipv4or6), v4);

    let v6: net::Ipv6Addr = "2001:0db8:0000:0000:0000:ff00:0042:8329".parse().unwrap();
    let ipv4or6: Ipv4or6 = v6.into();
    assert_eq!(net::Ipv6Addr::from(ipv4or6), v6);
}

#[test]
fn port_convert_u16() {
    let port = Port::from(1234);
    assert_eq!(u16::from(port), 1234);
}
