//! Transitional gateway port traits.
//!
//! These traits exist to connect a gateway runner to a concrete external
//! messaging implementation. They are gateway-internal transport ports, not
//! runtime abstractions and not the final product vocabulary.

use super::matrix::MatrixInboundEvent;
use super::GatewayOutboundDelivery;

pub trait GatewayTransportClient<Inbound, Delivery> {
    fn recv_inbound(&mut self) -> Result<Option<Inbound>, String>;

    fn send_delivery(&mut self, delivery: Delivery) -> Result<(), String>;
}

pub trait MatrixGatewayClient:
    GatewayTransportClient<MatrixInboundEvent, GatewayOutboundDelivery>
{
}

impl<T> MatrixGatewayClient for T where
    T: GatewayTransportClient<MatrixInboundEvent, GatewayOutboundDelivery>
{
}
