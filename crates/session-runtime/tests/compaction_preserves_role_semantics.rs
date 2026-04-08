use zstar_session_runtime::compaction::compact_with_summary;
use zstar_session_runtime::message_projection::{
    DurableTranscriptEntry, DurableTranscriptKind, SummaryArtifactRole,
};
use zstar_session_runtime::session::Session;
use zstar_session_runtime::sqlite::SqliteStorage;
use zstar_session_runtime::storage::TranscriptStore;
use zstar_session_runtime::RuntimeMessage;

#[test]
fn compaction_preserves_role_semantics() {
    let history = vec![
        RuntimeMessage::Assistant("first answer".to_string()),
        RuntimeMessage::ToolResult("result:alpha".to_string()),
        RuntimeMessage::Assistant("second answer".to_string()),
    ];
    let compacted = compact_with_summary(&history, "condensed prior context");

    assert_eq!(
        compacted.summary_artifact().role,
        SummaryArtifactRole::RuntimeSystem,
        "compaction summaries must be carried as explicit runtime/system artifacts rather than user-authored messages"
    );

    assert_eq!(
        compacted.compacted_context,
        vec![RuntimeMessage::RuntimeSummary(
            "condensed prior context".to_string()
        )],
        "compaction should inject an explicit runtime summary artifact into active context"
    );

    let tempdir = tempfile::tempdir().expect("temp dir");
    let db_path = tempdir.path().join("compaction.sqlite3");
    let mut storage = SqliteStorage::open(&db_path).expect("open sqlite storage");
    let mut session = Session::new(compacted.compacted_context.clone());
    session.record_compaction(compacted.record.clone());

    storage.persist_session(&session).expect("persist session");

    assert_eq!(
        storage.load_transcript().expect("load transcript"),
        vec![DurableTranscriptEntry {
            kind: DurableTranscriptKind::RuntimeSummary,
            content: "condensed prior context".to_string(),
        }],
        "compaction summaries must be persisted with runtime-summary semantics rather than a user-authored transcript role"
    );

    assert_eq!(
        storage
            .load_compaction_records()
            .expect("load compaction records"),
        vec![compacted.record.clone()],
        "compaction metadata must keep the pre-compaction messages recoverable"
    );
}
