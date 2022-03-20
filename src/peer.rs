use std::marker::PhantomData;
use std::time::Duration;

use citizen_enet_sys::{
    enet_peer_disconnect, enet_peer_disconnect_later, enet_peer_disconnect_now, enet_peer_receive,
    enet_peer_reset, enet_peer_send, enet_peer_throttle_configure, enet_peer_timeout, ENetPeer,
    _ENetPeerState,
    _ENetPeerState_ENET_PEER_STATE_DISCONNECTED,
    _ENetPeerState_ENET_PEER_STATE_CONNECTING,
    _ENetPeerState_ENET_PEER_STATE_ACKNOWLEDGING_CONNECT,
    _ENetPeerState_ENET_PEER_STATE_CONNECTION_PENDING,
    _ENetPeerState_ENET_PEER_STATE_CONNECTION_SUCCEEDED,
    _ENetPeerState_ENET_PEER_STATE_CONNECTED,
    _ENetPeerState_ENET_PEER_STATE_DISCONNECT_LATER,
    _ENetPeerState_ENET_PEER_STATE_DISCONNECTING,
    _ENetPeerState_ENET_PEER_STATE_ACKNOWLEDGING_DISCONNECT,
    _ENetPeerState_ENET_PEER_STATE_ZOMBIE,
};

use citizen_enet_sys::ENET_PEER_PACKET_THROTTLE_SCALE;

/// When the throttle has a value of ENET_PEER_PACKET_THROTTLE_SCALE,
/// no unreliable packets are dropped by ENet, and so 100% of all unreliable packets will be sent. 
pub static PACKET_THROTTLE_SCALE: u32 = ENET_PEER_PACKET_THROTTLE_SCALE as u32;

use crate::{Address, Error, Packet};

/// This struct represents an endpoint in an ENet-connection.
///
/// The lifetime of these instances is not really clear from the ENet documentation.
/// Therefore, `Peer`s are always borrowed, and can not really be stored anywhere.
///
/// ENet allows the association of arbitrary data with each peer.
/// The type of this associated data is chosen through `T`.
#[derive(Clone, Debug)]
pub struct Peer<'a, T: 'a> {
    inner: *mut ENetPeer,

    _data: PhantomData<&'a mut T>,
}

/// A packet received directly from a `Peer`.
///
/// Contains the received packet as well as the channel on which it was received.
#[derive(Debug)]
pub struct PeerPacket<'b, 'a, T: 'a> {
    /// The packet that was received.
    pub packet: Packet,
    /// The channel on which the packet was received.
    pub channel_id: u8,

    _priv_guard: PhantomData<&'b Peer<'a, T>>,
}

/// Describes the state a `Peer` is in.
///
/// The states should be self-explanatory, ENet doesn't explain them more either.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum PeerState {
    Disconnected,
    Connected,
    Connecting,
    AcknowledgingConnect,
    ConnectionPending,
    ConnectionSucceeded,
    DisconnectLater,
    Disconnecting,
    AcknowledgingDisconnect,
    Zombie,
}

impl PeerState {
    fn from_sys_state(citizen_enet_sys_state: _ENetPeerState) -> PeerState {
        #[allow(non_upper_case_globals)]
        match citizen_enet_sys_state {
            _ENetPeerState_ENET_PEER_STATE_DISCONNECTED => PeerState::Disconnected,
            _ENetPeerState_ENET_PEER_STATE_CONNECTING => PeerState::Connecting,
            _ENetPeerState_ENET_PEER_STATE_ACKNOWLEDGING_CONNECT => PeerState::AcknowledgingConnect,
            _ENetPeerState_ENET_PEER_STATE_CONNECTION_PENDING => PeerState::ConnectionPending,
            _ENetPeerState_ENET_PEER_STATE_CONNECTION_SUCCEEDED => PeerState::ConnectionSucceeded,
            _ENetPeerState_ENET_PEER_STATE_CONNECTED => PeerState::Connected,
            _ENetPeerState_ENET_PEER_STATE_DISCONNECT_LATER => PeerState::DisconnectLater,
            _ENetPeerState_ENET_PEER_STATE_DISCONNECTING => PeerState::Disconnecting,
            _ENetPeerState_ENET_PEER_STATE_ACKNOWLEDGING_DISCONNECT => PeerState::AcknowledgingDisconnect,
            _ENetPeerState_ENET_PEER_STATE_ZOMBIE => PeerState::Zombie,
            val => panic!("unexpected peer state: {}", val),
        }
    }
}

impl<'a, T> Peer<'a, T> {
    pub(crate) fn new(inner: *mut ENetPeer) -> Self {
        Self {
            inner,
            _data: PhantomData,
        }
    }

    /// Returns the address of this `Peer`.
    pub fn address(&self) -> Address {
        Address::from_enet_address(&unsafe { (*self.inner).address })
    }

    /// Returns the amount of channels allocated for this `Peer`.
    pub fn channel_count(&self) -> usize {
        unsafe { (*self.inner).channelCount }
    }

    /// Returns the data passed to connect by the peer
    pub fn event_data(&self) -> u32 {
        unsafe { (*self.inner).eventData }
    }

    /// Returns a reference to the data associated with this `Peer`, if set.
    pub fn data(&self) -> Option<&T> {
        unsafe {
            let raw_data = (*self.inner).data as *const T;

            if raw_data.is_null() {
                None
            } else {
                Some(&(*raw_data))
            }
        }
    }

    /// Returns a mutable reference to the data associated with this `Peer`, if set.
    pub fn data_mut(&mut self) -> Option<&mut T> {
        unsafe {
            let raw_data = (*self.inner).data as *mut T;

            if raw_data.is_null() {
                None
            } else {
                Some(&mut (*raw_data))
            }
        }
    }

    /// Sets or clears the data associated with this `Peer`, replacing existing data.
    pub fn set_data(&mut self, data: Option<T>) {
        unsafe {
            let raw_data = (*self.inner).data as *mut T;

            if !raw_data.is_null() {
                // free old data
                let _: Box<T> = Box::from_raw(raw_data);
            }

            let new_data = match data {
                Some(data) => Box::into_raw(Box::new(data)) as *mut _,
                None => std::ptr::null_mut(),
            };

            (*self.inner).data = new_data;
        }
    }

    /// Returns the downstream bandwidth of this `Peer` in bytes/second.
    pub fn incoming_bandwidth(&self) -> u32 {
        unsafe { (*self.inner).incomingBandwidth }
    }

    /// Returns the upstream bandwidth of this `Peer` in bytes/second.
    pub fn outgoing_bandwidth(&self) -> u32 {
        unsafe { (*self.inner).outgoingBandwidth }
    }

    /// Returns the mean round trip time between sending a reliable packet and receiving its acknowledgement.
    pub fn mean_rtt(&self) -> Duration {
        Duration::from_millis(unsafe { (*self.inner).roundTripTime } as u64)
    }

    /// Forcefully disconnects this `Peer`.
    ///
    /// The foreign host represented by the peer is not notified of the disconnection and will timeout on its connection to the local host.
    pub fn reset(self) {
        unsafe {
            enet_peer_reset(self.inner);
        }
    }

    /// Returns the state this `Peer` is in.
    pub fn state(&self) -> PeerState {
        PeerState::from_sys_state(unsafe {(*self.inner).state})
    }

    /// Configures throttle parameter for a peer. 
    pub fn configure_throttling(&mut self, interval: u32, acceleration: u32, deceleration: u32) {
        unsafe {
            enet_peer_throttle_configure(self.inner, interval, acceleration, deceleration);
        }
    }

    pub fn set_timeout(&mut self, limit: u32, min: u32, max: u32) {
        unsafe {
            enet_peer_timeout(self.inner, limit, min, max);
        }
    }

    /// Queues a packet to be sent.
    ///
    /// Actual sending will happen during `Host::service`.
    pub fn send_packet(&mut self, packet: Packet, channel_id: u8) -> Result<(), Error> {
        let res = unsafe { enet_peer_send(self.inner, channel_id, packet.into_inner()) };

        match res {
            r if r > 0 => panic!("unexpected res: {}", r),
            0 => Ok(()),
            r if r < 0 => Err(Error(r)),
            _ => panic!("unreachable"),
        }
    }

    /// Disconnects from this peer.
    ///
    /// A `Disconnect` event will be returned by `Host::service` once the disconnection is complete.
    pub fn disconnect(&mut self, user_data: u32) {
        unsafe {
            enet_peer_disconnect(self.inner, user_data);
        }
    }

    /// Disconnects from this peer immediately.
    ///
    /// No `Disconnect` event will be created. No disconnect notification for the foreign peer is guaranteed, and this `Peer` is immediately reset on return from this method.
    pub fn disconnect_now(self, user_data: u32) {
        unsafe {
            enet_peer_disconnect_now(self.inner, user_data);
        }
    }

    /// Disconnects from this peer after all outgoing packets have been sent.
    ///
    /// A `Disconnect` event will be returned by `Host::service` once the disconnection is complete.
    pub fn disconnect_later(&mut self, user_data: u32) {
        unsafe {
            enet_peer_disconnect_later(self.inner, user_data);
        }
    }

    /// Attempts to dequeue an incoming packet from this `Peer`.
    ///
    /// On success, returns the packet and the channel id of the receiving channel.
    pub fn receive<'b>(&'b mut self) -> Option<PeerPacket<'b, 'a, T>> {
        let mut channel_id = 0u8;

        let res = unsafe { enet_peer_receive(self.inner, &mut channel_id as *mut _) };

        if res.is_null() {
            return None;
        }

        Some(PeerPacket {
            packet: Packet::from_sys_packet(res),
            channel_id,
            _priv_guard: PhantomData,
        })
    }
}
