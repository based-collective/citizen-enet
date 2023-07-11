use std::ffi::CString;
use std::net::{Ipv4Addr, Ipv6Addr, IpAddr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::ops::{Deref, DerefMut};

use crate::Error;

use citizen_enet_sys::{ENetAddress, in6_addr, in6_addr__bindgen_ty_1};

/// An address that can be used with the ENet API.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Address(pub SocketAddr);

impl Address {
    /// Create a new address from a given hostname.
    pub fn from_hostname(hostname: &CString, port: u16) -> Result<Address, Error> {
        use citizen_enet_sys::enet_address_set_host;

        let host = unsafe { std::mem::transmute::<_, in6_addr>([0u8; 16]) };

        let mut addr = ENetAddress { host, port, sin6_scope_id: 0 };

        let res =
            unsafe { enet_address_set_host(&mut addr as *mut ENetAddress, hostname.as_ptr()) };

        if res != 0 {
            return Err(Error(res));
        }

        Ok(Self::from_enet_address(&addr))
    }

    /// Return the ip of this address
    pub fn ip(&self) -> IpAddr {
        self.0.ip()
    }

    /// Returns the port of this address
    pub fn port(&self) -> u16 {
        self.0.port()
    }

    pub(crate) fn enet_address(&self) -> ENetAddress {
        match self.0 {
            SocketAddr::V4(addr) => {
                let octets = addr.ip().octets();
                ENetAddress {
                    host: unsafe { std::mem::transmute::<_, in6_addr>([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0xFF, octets[0], octets[1], octets[2], octets[3]]) },
                    port: self.port(),
                    sin6_scope_id: 0,
                }
            },
            SocketAddr::V6(addr) => {
                ENetAddress {
                    host: unsafe { std::mem::transmute::<_, in6_addr>(addr.ip().octets()) },
                    port: self.port(),
                    sin6_scope_id: addr.scope_id() as u16
                }
            }
        }
    }

    pub(crate) fn from_enet_address(addr: &ENetAddress) -> Address {
        let hextets = &unsafe { std::mem::transmute::<_, [u16; 8]>(addr.host) };
        let octets = &unsafe { std::mem::transmute::<_, [u8; 16]>(addr.host) };
        if hextets[0] == 0 && hextets[1] == 0 && hextets[2] == 0 && hextets[3] == 0 && hextets[4] == 0 && hextets[5] == 0xFFFF {
            Address(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(octets[12], octets[13], octets[14], octets[15]), addr.port)))
        } else {
            Address(SocketAddr::V6(SocketAddrV6::new(
                Ipv6Addr::from(*hextets),
                addr.port,
                0,
                addr.sin6_scope_id as u32
            )))
        }
    }
}

impl Deref for Address {
    type Target = SocketAddr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Address {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<SocketAddr> for Address {
    fn from(addr: SocketAddr) -> Address {
        Address(addr)
    }
}

impl From<SocketAddrV4> for Address {
    fn from(addr: SocketAddrV4) -> Address {
        Address(SocketAddr::V4(addr))
    }
}

impl From<SocketAddrV6> for Address {
    fn from(addr: SocketAddrV6) -> Address {
        Address(SocketAddr::V6(addr))
    }
}

#[cfg(test)]
mod tests {
    use super::Address;

    use std::ffi::CString;
    use std::net::{Ipv4Addr, IpAddr};

    #[test]
    fn test_from_valid_hostname() {
        let addr = Address::from_hostname(&CString::new("localhost").unwrap(), 0).unwrap();
        assert_eq!(addr.ip(), IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        assert_eq!(addr.port(), 0);
    }

    #[test]
    fn test_from_invalid_hostname() {
        assert!(Address::from_hostname(&CString::new("").unwrap(), 0).is_err());
    }
}
