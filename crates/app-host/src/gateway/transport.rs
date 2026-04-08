use std::path::{Path, PathBuf};

use zstar_agent_core::provider::Provider;

use super::client::MatrixGatewayClient;
use super::matrix::{project_matrix_streamed_delivery, MatrixOutboundRoute};
use super::runtime::{
    GatewayDeliveryRetry, GatewayProcessOutcome, GatewayRunStatus, GatewayRuntime,
};
use super::{GatewayOutboundDelivery, GatewayThread};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatrixTransportConfig {
    pub model: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatrixTransportCycleReport {
    pub status: GatewayRunStatus,
    pub session_id: Option<String>,
    pub deliveries_sent: usize,
    pub retries_recorded: usize,
}

#[derive(Debug, Clone)]
pub struct MatrixGatewayTransport {
    home: PathBuf,
    config: MatrixTransportConfig,
}

impl MatrixGatewayTransport {
    pub fn new(home: impl AsRef<Path>, config: MatrixTransportConfig) -> Self {
        Self {
            home: home.as_ref().to_path_buf(),
            config,
        }
    }

    pub async fn run_once(
        &self,
        client: &mut dyn MatrixGatewayClient,
        provider: &mut dyn Provider,
    ) -> Result<Option<MatrixTransportCycleReport>, String> {
        let Some(event) = client.recv_inbound()? else {
            return Ok(None);
        };

        let mut runtime =
            GatewayRuntime::load_or_default(&self.home).map_err(|error| error.to_string())?;
        let outcome = runtime
            .process_matrix_event(&self.home, &self.config.model, &event, provider)
            .await?;
        let report = match outcome.status {
            GatewayRunStatus::Duplicate => MatrixTransportCycleReport {
                status: GatewayRunStatus::Duplicate,
                session_id: None,
                deliveries_sent: 0,
                retries_recorded: 0,
            },
            GatewayRunStatus::Processed => {
                let (deliveries_sent, retries_recorded) =
                    self.deliver_live_run(&mut runtime, client, &event, &outcome)?;
                MatrixTransportCycleReport {
                    status: GatewayRunStatus::Processed,
                    session_id: outcome.session_id.clone(),
                    deliveries_sent,
                    retries_recorded,
                }
            }
        };

        runtime
            .save(&self.home)
            .map_err(|error| error.to_string())?;
        Ok(Some(report))
    }

    pub fn flush_pending_retries(
        &self,
        client: &mut dyn MatrixGatewayClient,
    ) -> Result<usize, String> {
        let mut runtime =
            GatewayRuntime::load_or_default(&self.home).map_err(|error| error.to_string())?;
        let pending = runtime.drain_pending_retries();
        let mut delivered = 0_usize;

        for retry in pending {
            let delivery = GatewayOutboundDelivery {
                kind: retry.kind.clone(),
                channel_id: retry.channel_id.clone(),
                thread: Some(GatewayThread {
                    session_id: fallback_retry_session_id(&retry),
                    thread_id: retry.thread_id.clone(),
                }),
                reply_to: retry.reply_to.clone(),
                body: retry.body.clone(),
            };

            if client.send_delivery(delivery).is_ok() {
                delivered += 1;
                continue;
            }

            runtime.record_retry(retry)?;
        }

        runtime
            .save(&self.home)
            .map_err(|error| error.to_string())?;
        Ok(delivered)
    }

    fn deliver_live_run(
        &self,
        runtime: &mut GatewayRuntime,
        client: &mut dyn MatrixGatewayClient,
        event: &super::matrix::MatrixInboundEvent,
        outcome: &GatewayProcessOutcome,
    ) -> Result<(usize, usize), String> {
        let Some(live_run) = outcome.live_run.as_ref() else {
            return Ok((0, 0));
        };

        let route = MatrixOutboundRoute {
            room_id: event.room_id.clone(),
            thread: Some(GatewayThread {
                session_id: outcome
                    .session_id
                    .clone()
                    .unwrap_or_else(|| fallback_session_id_from_event(event)),
                thread_id: event.thread_id.clone(),
            }),
            reply_to: event.event_id.clone(),
        };
        let deliveries = project_matrix_streamed_delivery(route, &live_run.events);
        let mut deliveries_sent = 0_usize;
        let mut retries_recorded = 0_usize;

        for delivery in deliveries {
            if client.send_delivery(delivery.clone()).is_ok() {
                deliveries_sent += 1;
                continue;
            }

            runtime.record_retry(GatewayDeliveryRetry {
                gateway_kind: "matrix".to_string(),
                kind: delivery.kind.clone(),
                channel_id: delivery.channel_id,
                thread_id: delivery.thread.and_then(|thread| thread.thread_id),
                reply_to: delivery.reply_to,
                body: delivery.body,
            })?;
            retries_recorded += 1;
        }

        Ok((deliveries_sent, retries_recorded))
    }
}

fn fallback_session_id_from_event(event: &super::matrix::MatrixInboundEvent) -> String {
    match event
        .thread_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(thread_id) => format!("matrix:{}:{}", event.room_id.trim(), thread_id),
        None => format!("matrix:{}", event.room_id.trim()),
    }
}

fn fallback_retry_session_id(retry: &GatewayDeliveryRetry) -> String {
    match retry
        .thread_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(thread_id) => format!("matrix:{}:{}", retry.channel_id.trim(), thread_id),
        None => format!("matrix:{}", retry.channel_id.trim()),
    }
}
