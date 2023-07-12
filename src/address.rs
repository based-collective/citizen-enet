use std::ffi::CString;
use std::net::{Ipv4Addr, Ipv6Addr, IpAddr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::ops::{Deref, DerefMut};

use crate::Error;

use citizen_enet_sys::{ENetAddress, in6_addr, in6_addr__bindgen_ty_1, enet_address_get_host_ip};

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
                let hextets = addr.ip().segments();
                let mut network_octets = [0u8; 16];
                network_octets[0] = (hextets[0] >> 8) as u8;
                network_octets[1] = hextets[0] as u8;
                network_octets[2] = (hextets[1] >> 8) as u8;
                network_octets[3] = hextets[1] as u8;
                network_octets[4] = (hextets[2] >> 8) as u8;
                network_octets[5] = hextets[2] as u8;
                network_octets[6] = (hextets[3] >> 8) as u8;
                network_octets[7] = hextets[3] as u8;
                network_octets[8] = (hextets[4] >> 8) as u8;
                network_octets[9] = hextets[4] as u8;
                network_octets[10] = (hextets[5] >> 8) as u8;
                network_octets[11] = hextets[5] as u8;
                network_octets[12] = (hextets[6] >> 8) as u8;
                network_octets[13] = hextets[6] as u8;
                network_octets[14] = (hextets[7] >> 8) as u8;
                network_octets[15] = hextets[7] as u8;
                ENetAddress {
                    host: unsafe { std::mem::transmute::<_, in6_addr>(network_octets) },
                    port: self.port(),
                    sin6_scope_id: addr.scope_id() as u16
                }
            }
        }
    }

    pub(crate) fn from_enet_address(addr: &ENetAddress) -> Address {
        // TODO: rename to segments, see Ipv6Addr::segments
        let hextets = &unsafe { std::mem::transmute::<_, [u16; 8]>(addr.host) };
        let octets = &unsafe { std::mem::transmute::<_, [u8; 16]>(addr.host) };
        if hextets[0] == 0 && hextets[1] == 0 && hextets[2] == 0 && hextets[3] == 0 && hextets[4] == 0 && hextets[5] == 0xFFFF {
            Address(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(octets[12], octets[13], octets[14], octets[15]), addr.port)))
        } else {
            let mut network_octets = [0u8; 16];
            network_octets[1] = (hextets[0] >> 8) as u8;
            network_octets[0] = hextets[0] as u8;
            network_octets[3] = (hextets[1] >> 8) as u8;
            network_octets[2] = hextets[1] as u8;
            network_octets[5] = (hextets[2] >> 8) as u8;
            network_octets[4] = hextets[2] as u8;
            network_octets[7] = (hextets[3] >> 8) as u8;
            network_octets[6] = hextets[3] as u8;
            network_octets[9] = (hextets[4] >> 8) as u8;
            network_octets[8] = hextets[4] as u8;
            network_octets[11] = (hextets[5] >> 8) as u8;
            network_octets[10] = hextets[5] as u8;
            network_octets[13] = (hextets[6] >> 8) as u8;
            network_octets[12] = hextets[6] as u8;
            network_octets[15] = (hextets[7] >> 8) as u8;
            network_octets[14] = hextets[7] as u8;
            Address(SocketAddr::V6(SocketAddrV6::new(
                Ipv6Addr::from(network_octets),
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
