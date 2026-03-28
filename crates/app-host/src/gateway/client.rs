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
