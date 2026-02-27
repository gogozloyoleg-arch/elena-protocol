//! Сетевой слой на libp2p: TCP/QUIC, Noise, Yamux, Floodsub.

mod p2p;

pub use p2p::{NetworkEvent, P2PNode};
