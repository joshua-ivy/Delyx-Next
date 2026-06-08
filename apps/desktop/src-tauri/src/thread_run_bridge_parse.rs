use crate::threads::{MessageRole, ThreadError, ThreadStatus};

pub(crate) fn parse_thread_status(status: &str) -> Result<ThreadStatus, ThreadError> {
    match status {
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

pub(crate) fn parse_message_role(role: &str) -> Result<MessageRole, ThreadError> {
    match role {
        "assistant" => Ok(MessageRole::Assistant),
        "system" => Ok(MessageRole::System),
        "user" => Ok(MessageRole::User),
        _ => Err(ThreadError::InvalidTransition),
    }
}
