use std::marker::PhantomData;

use citizen_enet_sys::{enet_socket_send, ENetSocket};

use crate::{Address, Error};

/// TODO: documentation
/// TODO: lifetimes
#[derive(Clone, Debug)]
pub struct Socket<'a, T: 'a> {
    inner: *mut u64,

    _data: PhantomData<&'a mut T>,
}

impl<'a, T> Socket<'a, T> {
    pub(crate) fn new(inner: *mut u64) -> Self {
        Self {
            inner,
            _data: PhantomData,
        }
    }

    pub fn send_data(&mut self, addr: &Address, data: &[u8]) -> Result<u32, Error> {
        let bytes_sent = unsafe {
            enet_socket_send(self.inner as ENetSocket, &addr.to_enet_address(), data.as_ptr() as *const _, data.len())
        };

        if bytes_sent < 0 {
            Err(Error(bytes_sent))
        } else {
            Ok(bytes_sent as u32)
        }
    }
}