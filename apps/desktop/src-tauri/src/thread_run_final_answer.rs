use crate::test_runner_bridge::{
    test_snapshot_from_store, TestRunnerBridgeState, TestRunnerBridgeStore,
};
use crate::thread_run_bridge::{ThreadRunBridgeState, ThreadRunStore};
use crate::thread_run_bridge_views::{record_view, ThreadRunRecordView, ThreadRunViewContext};
use crate::threads::ThreadStatus;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadFinalAnswerRequest {
    pub thread_id: String,
    pub summary: String,
    pub updated_at: String,
}

#[tauri::command]
pub fn thread_final_answer_record(
    state: tauri::State<ThreadRunBridgeState>,
    tests: tauri::State<TestRunnerBridgeState>,
    request: ThreadFinalAnswerRequest,
) -> Result<ThreadRunRecordView, String> {
    let run_id = {
        let store = state
            .store
            .lock()
            .map_err(|_| "Thread bridge lock failed.".to_string())?;
        run_id_for_thread(&store, &request.thread_id)?
    };
    let test_ids = tests.with_store(|store| passed_test_ids(store, &run_id))?;
    let mut store = state
        .store
        .lock()
        .map_err(|_| "Thread bridge lock failed.".to_string())?;
    let view = finalize_thread_record(&mut store, request, test_ids)?;
    state.persist(&store)?;
    Ok(view)
}

pub fn finalize_thread_record(
    store: &mut ThreadRunStore,
    request: ThreadFinalAnswerRequest,
    test_artifact_ids: Vec<String>,
) -> Result<ThreadRunRecordView, String> {
    let summary = request.summary.trim();
    if request.thread_id.trim().is_empty() || summary.is_empty() {
        return Err("Final answer requires a thread ID and summary.".to_string());
    }
    validate_thread_can_finish(store, &request.thread_id)?;
    let run_id = run_id_for_thread(store, &request.thread_id)?;
    let evidence_ids = evidence_ids_for_run(store, &run_id)?;
    let test_ids = unique_non_empty(test_artifact_ids);
    let message = format!(
        "Final answer support linked {} evidence record(s) and {} passed test artifact(s).",
        evidence_ids.len(),
        test_ids.len()
    );
    store
        .ledger
        .append_event(&run_id, "final_answer.support_synthesized", &message)
        .map_err(|error| format!("{error:?}"))?;
    store
        .ledger
        .complete_run_with_support(&run_id, summary, evidence_ids, test_ids)
        .map_err(|error| format!("{error:?}"))?;
    store
        .manager
        .set_status(&request.thread_id, ThreadStatus::Done)
        .map_err(|error| format!("{error:?}"))?;
    let record = store
        .records
        .iter_mut()
        .find(|record| record.thread_id == request.thread_id)
        .ok_or_else(|| "Thread run record was not found.".to_string())?;
    record.updated_at = request.updated_at;
    let context = ThreadRunViewContext {
        created_at: record.created_at.clone(),
        project_id: record.project_id.clone(),
        run_id: record.run_id.clone(),
        updated_at: record.updated_at.clone(),
    };
    let thread = store
        .manager
        .get_thread(&request.thread_id)
        .map_err(|error| format!("{error:?}"))?;
    let run = store
        .ledger
        .get_run(&context.run_id)
        .map_err(|error| format!("{error:?}"))?;
    Ok(record_view(thread, run, &context))
}

pub fn passed_test_ids(store: &TestRunnerBridgeStore, run_id: &str) -> Vec<String> {
    test_snapshot_from_store(store, run_id)
        .into_iter()
        .filter(|artifact| artifact.status == "passed")
        .map(|artifact| artifact.id)
        .collect()
}

fn validate_thread_can_finish(store: &ThreadRunStore, thread_id: &str) -> Result<(), String> {
    let thread = store
        .manager
        .get_thread(thread_id)
        .map_err(|error| format!("{error:?}"))?;
    if thread.archived || matches!(thread.status, ThreadStatus::Done | ThreadStatus::Failed) {
        return Err("Final answer cannot finish an archived or terminal thread.".to_string());
    }
    Ok(())
}

fn run_id_for_thread(store: &ThreadRunStore, thread_id: &str) -> Result<String, String> {
    store
        .records
        .iter()
        .find(|record| record.thread_id == thread_id)
        .map(|record| record.run_id.clone())
        .ok_or_else(|| "Thread run record was not found.".to_string())
}

fn evidence_ids_for_run(store: &ThreadRunStore, run_id: &str) -> Result<Vec<String>, String> {
    let run = store
        .ledger
        .get_run(run_id)
        .map_err(|error| format!("{error:?}"))?;
    Ok(unique_non_empty(
        run.evidence.iter().map(|item| item.id.clone()),
    ))
}

fn unique_non_empty(items: impl IntoIterator<Item = String>) -> Vec<String> {
    items.into_iter().fold(Vec::new(), |mut acc, item| {
        if !item.trim().is_empty() && !acc.contains(&item) {
            acc.push(item);
        }
        acc
    })
}
