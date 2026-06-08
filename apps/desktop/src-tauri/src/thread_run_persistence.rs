use crate::agent_run_persistence::{load_from_connection, save_to_connection};
use crate::thread_run_bridge::{ThreadRunRecord, ThreadRunStore};
use crate::threads::{
    MessageRole, TaskThread, ThreadError, ThreadManager, ThreadMessage, ThreadStatus,
};
use rusqlite::{params, Connection};
use std::path::Path;

pub fn save_to_path(store: &ThreadRunStore, path: &Path) -> Result<(), String> {
    let mut connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    let transaction = connection.transaction().map_err(sql_string)?;
    clear_thread_tables(&transaction)?;
    save_to_connection(&store.ledger, &transaction).map_err(|error| format!("{error:?}"))?;
    for thread in store.manager.all_threads() {
        insert_thread(&transaction, thread)?;
        for (index, message) in thread.messages.iter().enumerate() {
            insert_message(&transaction, &thread.id, index, message)?;
        }
    }
    for record in &store.records {
        insert_record(&transaction, record)?;
    }
    transaction.commit().map_err(sql_string)
}

pub fn load_from_path(path: &Path) -> Result<ThreadRunStore, String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    let ledger = load_from_connection(&connection).map_err(|error| format!("{error:?}"))?;
    let threads = load_threads(&connection)?;
    let records = load_records(&connection)?;
    Ok(ThreadRunStore {
        manager: ThreadManager::from_loaded_threads(threads),
        ledger,
        records,
    })
}

fn clear_thread_tables(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(
            "DELETE FROM thread_run_records;
             DELETE FROM thread_messages;
             DELETE FROM task_threads;",
        )
        .map_err(sql_string)
}

fn insert_thread(connection: &Connection, thread: &TaskThread) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO task_threads (id, project_id, title, goal, status, archived) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![thread.id, thread.project_id, thread.title, thread.goal, status_key(thread.status), thread.archived as i32],
        )
        .map(|_| ())
        .map_err(sql_string)
}

fn insert_message(
    connection: &Connection,
    thread_id: &str,
    index: usize,
    message: &ThreadMessage,
) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO thread_messages (thread_id, message_index, role, body) VALUES (?1, ?2, ?3, ?4)",
            params![thread_id, index as i64, role_key(message.role), message.body],
        )
        .map(|_| ())
        .map_err(sql_string)
}

fn insert_record(connection: &Connection, record: &ThreadRunRecord) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO thread_run_records (thread_id, run_id, project_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![record.thread_id, record.run_id, record.project_id, record.created_at, record.updated_at],
        )
        .map(|_| ())
        .map_err(sql_string)
}

fn load_threads(connection: &Connection) -> Result<Vec<TaskThread>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, project_id, title, goal, status, archived FROM task_threads ORDER BY rowid",
        )
        .map_err(sql_string)?;
    let mut rows = statement.query([]).map_err(sql_string)?;
    let mut threads = Vec::new();
    while let Some(row) = rows.next().map_err(sql_string)? {
        let id: String = row.get(0).map_err(sql_string)?;
        let status_value: String = row.get(4).map_err(sql_string)?;
        threads.push(TaskThread {
            id: id.clone(),
            project_id: row.get(1).map_err(sql_string)?,
            title: row.get(2).map_err(sql_string)?,
            goal: row.get(3).map_err(sql_string)?,
            status: parse_status(&status_value).map_err(|error| format!("{error:?}"))?,
            messages: load_messages(connection, &id)?,
            archived: row.get::<_, i64>(5).map_err(sql_string)? != 0,
        });
    }
    Ok(threads)
}

fn load_messages(connection: &Connection, thread_id: &str) -> Result<Vec<ThreadMessage>, String> {
    let mut statement = connection
        .prepare(
            "SELECT role, body FROM thread_messages WHERE thread_id = ?1 ORDER BY message_index",
        )
        .map_err(sql_string)?;
    let mut rows = statement.query(params![thread_id]).map_err(sql_string)?;
    let mut messages = Vec::new();
    while let Some(row) = rows.next().map_err(sql_string)? {
        let role_value: String = row.get(0).map_err(sql_string)?;
        messages.push(ThreadMessage {
            role: parse_role(&role_value).map_err(|error| format!("{error:?}"))?,
            body: row.get(1).map_err(sql_string)?,
        });
    }
    Ok(messages)
}

fn load_records(connection: &Connection) -> Result<Vec<ThreadRunRecord>, String> {
    let mut statement = connection
        .prepare("SELECT thread_id, run_id, project_id, created_at, updated_at FROM thread_run_records ORDER BY rowid")
        .map_err(sql_string)?;
    let rows = statement
        .query_map([], |row| {
            Ok(ThreadRunRecord {
                thread_id: row.get(0)?,
                run_id: row.get(1)?,
                project_id: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })
        .map_err(sql_string)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(sql_string)
}

fn status_key(status: ThreadStatus) -> &'static str {
    match status {
        ThreadStatus::Blocked => "blocked",
        ThreadStatus::Building => "building",
        ThreadStatus::Done => "done",
        ThreadStatus::Exploring => "exploring",
        ThreadStatus::Failed => "failed",
        ThreadStatus::Idle => "idle",
        ThreadStatus::Planning => "planning",
        ThreadStatus::Reviewing => "reviewing",
        ThreadStatus::Testing => "testing",
        ThreadStatus::WaitingForApproval => "waiting_for_approval",
    }
}

fn parse_status(value: &str) -> Result<ThreadStatus, ThreadError> {
    match value {
        "blocked" => Ok(ThreadStatus::Blocked),
        "building" => Ok(ThreadStatus::Building),
        "done" => Ok(ThreadStatus::Done),
        "exploring" => Ok(ThreadStatus::Exploring),
        "failed" => Ok(ThreadStatus::Failed),
        "idle" => Ok(ThreadStatus::Idle),
        "planning" => Ok(ThreadStatus::Planning),
        "reviewing" => Ok(ThreadStatus::Reviewing),
        "testing" => Ok(ThreadStatus::Testing),
        "waiting_for_approval" => Ok(ThreadStatus::WaitingForApproval),
        _ => Err(ThreadError::InvalidTransition),
    }
}

fn role_key(role: MessageRole) -> &'static str {
    match role {
        MessageRole::Assistant => "assistant",
        MessageRole::System => "system",
        MessageRole::User => "user",
    }
}

fn parse_role(value: &str) -> Result<MessageRole, ThreadError> {
    match value {
        "assistant" => Ok(MessageRole::Assistant),
        "system" => Ok(MessageRole::System),
        "user" => Ok(MessageRole::User),
        _ => Err(ThreadError::InvalidTransition),
    }
}

fn sql_string(error: rusqlite::Error) -> String {
    error.to_string()
}
