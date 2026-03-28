use std::io;
use std::path::Path;

use matrixclaw_agent_core::provider::Provider;

use super::matrix::{normalize_matrix_inbound_event, MatrixInboundEvent};
use super::store::{GatewayDeliveryRetryRecord, GatewaySessionStore};
use super::OutboundDeliveryKind;
use crate::ingress::run_ingress_with_provider;
use crate::live_runtime::LiveRunOutcome;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayDeliveryRetry {
    pub gateway_kind: String,
    pub kind: OutboundDeliveryKind,
    pub channel_id: String,
    pub thread_id: Option<String>,
    pub reply_to: Option<String>,
    pub body: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GatewayRunStatus {
    Processed,
    Duplicate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayProcessOutcome {
    pub status: GatewayRunStatus,
    pub session_id: Option<String>,
    pub live_run: Option<LiveRunOutcome>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GatewayRuntime {
    processed_inbound_event_ids: Vec<String>,
    pending_retries: Vec<GatewayDeliveryRetry>,
}

impl GatewayRuntime {
    pub fn load_or_default(home: impl AsRef<Path>) -> io::Result<Self> {
        let store = GatewaySessionStore::load_or_default(home)?;
        Ok(Self {
            processed_inbound_event_ids: store.processed_inbound_event_ids,
            pending_retries: store
                .pending_delivery_retries
                .into_iter()
                .map(GatewayDeliveryRetry::from_record)
                .collect(),
        })
    }

    pub fn save(&self, home: impl AsRef<Path>) -> io::Result<()> {
        let mut store = GatewaySessionStore::load_or_default(&home)?;
        store.processed_inbound_event_ids = self.processed_inbound_event_ids.clone();
        store.pending_delivery_retries = self
            .pending_retries
            .iter()
            .cloned()
            .map(GatewayDeliveryRetry::into_record)
            .collect();
        let _ = store.save(home)?;
        Ok(())
    }

    pub fn process_matrix_event(
        &mut self,
        home: impl AsRef<Path>,
        model: impl Into<String>,
        event: &MatrixInboundEvent,
        provider: &mut dyn Provider,
    ) -> Result<GatewayProcessOutcome, String> {
        if let Some(event_id) = event.event_id.as_deref() {
            if self
                .processed_inbound_event_ids
                .iter()
                .any(|seen| seen == event_id)
            {
                return Ok(GatewayProcessOutcome {
                    status: GatewayRunStatus::Duplicate,
                    session_id: None,
                    live_run: None,
                });
            }
        }

        let envelope = normalize_matrix_inbound_event(home.as_ref(), event)?;
        let outcome = run_ingress_with_provider(home, model, &envelope, provider)?;
        if let Some(event_id) = event.event_id.as_deref() {
            self.processed_inbound_event_ids.push(event_id.to_string());
        }
        Ok(GatewayProcessOutcome {
            status: GatewayRunStatus::Processed,
            session_id: Some(outcome.live_run.session_id.clone()),
            live_run: Some(outcome.live_run),
        })
    }

    pub fn record_retry(&mut self, retry: GatewayDeliveryRetry) -> Result<(), String> {
        if retry.gateway_kind.trim().is_empty() {
            return Err("gateway_kind is required".to_string());
        }
        if retry.channel_id.trim().is_empty() {
            return Err("channel_id is required".to_string());
        }
        if retry.body.trim().is_empty() {
            return Err("body is required".to_string());
        }
        self.pending_retries.push(retry);
        Ok(())
    }

    pub fn pending_retries(&self) -> &[GatewayDeliveryRetry] {
        &self.pending_retries
    }

    pub fn drain_pending_retries(&mut self) -> Vec<GatewayDeliveryRetry> {
        std::mem::take(&mut self.pending_retries)
    }
}

impl GatewayDeliveryRetry {
    fn into_record(self) -> GatewayDeliveryRetryRecord {
        GatewayDeliveryRetryRecord {
            gateway_kind: self.gateway_kind,
            kind: self.kind,
            channel_id: self.channel_id,
            thread_id: self.thread_id,
            reply_to: self.reply_to,
            body: self.body,
        }
    }

    fn from_record(record: GatewayDeliveryRetryRecord) -> Self {
        Self {
            gateway_kind: record.gateway_kind,
            kind: record.kind,
            channel_id: record.channel_id,
            thread_id: record.thread_id,
            reply_to: record.reply_to,
            body: record.body,
        }
    }
}
