use matrixclaw_session_runtime::message_projection::{
    DurableTranscriptEntry, DurableTranscriptKind,
};
use matrixclaw_session_runtime::sqlite::SqliteStorage;
use matrixclaw_session_runtime::storage::TranscriptStore;
use matrixclaw_session_runtime::RuntimeMessage;

#[test]
fn transcript_matches_visible_behavior() {
    let tempdir = tempfile::tempdir().expect("temp dir");
    let db_path = tempdir.path().join("transcript.sqlite3");
    let mut storage = SqliteStorage::open(&db_path).expect("open sqlite storage");

    let history = vec![
        RuntimeMessage::Assistant("let me check".to_string()),
        RuntimeMessage::ToolResult("result:5".to_string()),
        RuntimeMessage::Warning("retrying after overflow".to_string()),
        RuntimeMessage::RetryMarker("retry #1".to_string()),
        RuntimeMessage::Assistant("final answer".to_string()),
    ];

    storage
        .persist_runtime_messages(&history)
        .expect("persist transcript");

    let stored = storage.load_transcript().expect("load transcript");

    assert_eq!(
        stored,
        vec![
            DurableTranscriptEntry {
                kind: DurableTranscriptKind::Assistant,
                content: "let me check".to_string(),
            },
            DurableTranscriptEntry {
                kind: DurableTranscriptKind::ToolResult,
                content: "result:5".to_string(),
            },
            DurableTranscriptEntry {
                kind: DurableTranscriptKind::Warning,
                content: "retrying after overflow".to_string(),
            },
            DurableTranscriptEntry {
                kind: DurableTranscriptKind::RetryMarker,
                content: "retry #1".to_string(),
            },
            DurableTranscriptEntry {
                kind: DurableTranscriptKind::Assistant,
                content: "final answer".to_string(),
            },
        ],
        "durable transcript must exactly mirror the user-visible conversation"
    );
}
