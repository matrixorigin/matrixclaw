use zstar_session_runtime::compaction::{CompactionRequest, CompactionResult, Compactor};
use zstar_session_runtime::compaction_record::CompactionRecord;
use zstar_session_runtime::error::RunFailure;
use zstar_session_runtime::run_controller::RunController;
use zstar_session_runtime::RuntimeMessage;

#[derive(Default)]
struct RecordingCompactor {
    requests: Vec<CompactionRequest>,
}

impl Compactor for RecordingCompactor {
    fn compact(&mut self, request: &CompactionRequest) -> CompactionResult {
        self.requests.push(request.clone());
        CompactionResult {
            compacted_context: vec![
                RuntimeMessage::Assistant("summary context".to_string()),
                RuntimeMessage::ToolResult("result:latest".to_string()),
            ],
            summary: "compacted overflow context".to_string(),
            record: CompactionRecord::new(
                "compacted overflow context",
                request.active_context.clone(),
            ),
        }
    }
}

#[test]
fn compacts_before_retry_on_overflow() {
    let mut controller = RunController::new(vec![
        RuntimeMessage::Assistant("initial answer".to_string()),
        RuntimeMessage::ToolResult("result:latest".to_string()),
        RuntimeMessage::Warning("overflow while preparing retry".to_string()),
    ]);
    let failure = RunFailure::overflow("context window exceeded");
    let mut compactor = RecordingCompactor::default();

    let outcome = controller
        .handle_run_failure(&failure, &mut compactor)
        .expect("overflow should produce a retry outcome");

    assert_eq!(
        compactor.requests.len(),
        1,
        "overflow handling should invoke compaction outside the core loop before retrying"
    );
    assert_eq!(
        compactor.requests[0].active_context,
        vec![
            RuntimeMessage::Assistant("initial answer".to_string()),
            RuntimeMessage::ToolResult("result:latest".to_string()),
        ],
        "failure-only warning context should be removed before compaction if it is not part of the next retry context"
    );
    assert_eq!(
        outcome.retried_context,
        vec![
            RuntimeMessage::Assistant("summary context".to_string()),
            RuntimeMessage::ToolResult("result:latest".to_string()),
        ],
        "retry should start from the compacted context rather than the pre-overflow context"
    );
    assert_eq!(
        outcome
            .compaction
            .expect("expected compaction metadata for overflow retry")
            .summary,
        "compacted overflow context".to_string()
    );
}
