use citizen_enet_sys::ENetBuffer;
use std::{marker::PhantomData, ffi::c_void};

use citizen_enet_sys::{enet_socket_send, ENetSocket};

use crate::{Address, Error};

// TODO: documentation
// TODO: lifetimes
#[derive(Clone, Debug)]
pub struct Socket<'a, T: 'a> {
    inner: ENetSocket,

    _data: PhantomData<&'a mut T>,
}

impl<'a, T> Socket<'a, T> {
    pub(crate) fn new(inner: ENetSocket) -> Self {
        Self {
            inner,
            _data: PhantomData,
        }
    }

    pub fn send_data(&mut self, addr: &Address, data: &[u8]) -> Result<u32, Error> {
        let bytes_sent = unsafe {
            let buffer = ENetBuffer {
                data: data.as_ptr() as *mut c_void,
                dataLength: data.len(),
            };
            enet_socket_send(self.inner, &addr.to_enet_address(), &buffer as *const _, 1)
        };

        if bytes_sent < 0 {
            Err(Error(bytes_sent))
        } else {
            Ok(bytes_sent as u32)
        }
    }
}