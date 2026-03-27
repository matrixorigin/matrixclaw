use matrixclaw_session_runtime::compaction_record::CompactionRecord;
use matrixclaw_session_runtime::recovery::{restore_session, SessionRecoveryStore};
use matrixclaw_session_runtime::session::Session;
use matrixclaw_session_runtime::sqlite::SqliteStorage;
use matrixclaw_session_runtime::RuntimeMessage;

#[test]
fn session_resume_after_restart() {
    let tempdir = tempfile::tempdir().expect("temp dir");
    let db_path = tempdir.path().join("session.sqlite3");
    let mut storage = SqliteStorage::open(&db_path).expect("open sqlite storage");

    let mut session = Session::new(vec![
        RuntimeMessage::Assistant("checking state".to_string()),
        RuntimeMessage::ToolResult("result:5".to_string()),
        RuntimeMessage::Warning("retrying after overflow".to_string()),
        RuntimeMessage::RetryMarker("retry #1".to_string()),
    ]);
    session.queue_steering_message("keep focus");
    session.queue_follow_up_message("answer the follow-up");
    session.record_compaction(CompactionRecord::new(
        "compacted older context",
        vec![RuntimeMessage::Assistant("previous answer".to_string())],
    ));

    storage.persist_session(&session).expect("persist session");

    let recovered = restore_session(
        storage
            .load_recovery_snapshot()
            .expect("load recovery snapshot"),
    );

    assert_eq!(
        recovered.session.history(),
        session.history(),
        "restarted session should recover the stored transcript"
    );

    assert_eq!(
        recovered.session.queue().items(),
        session.queue().items(),
        "restarted session should reconstruct queued runtime metadata"
    );

    assert_eq!(
        recovered.session.compaction_records(),
        session.compaction_records(),
        "restarted session should recover persisted compaction provenance"
    );

    assert_eq!(
        recovered.context.next_turn,
        session.project_next_turn(),
        "next prompt should continue from the persisted state"
    );

    assert_eq!(
        recovered.context.next_run,
        session.project_next_run(),
        "next run context should include deferred follow-up delivery"
    );
}
