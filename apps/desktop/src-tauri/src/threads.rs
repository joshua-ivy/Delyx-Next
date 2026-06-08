#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskThread {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub goal: String,
    pub status: ThreadStatus,
    pub messages: Vec<ThreadMessage>,
    pub archived: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadStatus {
    Idle,
    Exploring,
    Planning,
    WaitingForApproval,
    Building,
    Testing,
    Reviewing,
    Blocked,
    Failed,
    Done,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadMessage {
    pub role: MessageRole,
    pub body: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThreadError {
    EmptyGoal,
    ProjectNotLinked,
    ThreadNotFound,
    InvalidTransition,
    ArchivedThread,
}

#[derive(Debug, Default)]
pub struct ThreadManager {
    project_ids: Vec<String>,
    threads: Vec<TaskThread>,
    next_id: usize,
}

impl ThreadManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn link_project(&mut self, project_id: impl Into<String>) {
        let project_id = project_id.into();
        if !self.project_ids.contains(&project_id) {
            self.project_ids.push(project_id);
        }
    }

    pub fn create_thread(
        &mut self,
        project_id: &str,
        goal: &str,
    ) -> Result<TaskThread, ThreadError> {
        if !self.project_ids.iter().any(|id| id == project_id) {
            return Err(ThreadError::ProjectNotLinked);
        }

        let goal = goal.trim();
        if goal.is_empty() {
            return Err(ThreadError::EmptyGoal);
        }

        self.next_id += 1;
        let thread = TaskThread {
            id: format!("{project_id}-thread-{}", self.next_id),
            project_id: project_id.to_string(),
            title: title_from_goal(goal),
            goal: goal.to_string(),
            status: ThreadStatus::Idle,
            messages: vec![ThreadMessage {
                role: MessageRole::User,
                body: goal.to_string(),
            }],
            archived: false,
        };

        self.threads.push(thread.clone());
        Ok(thread)
    }

    pub fn list_threads(&self, project_id: &str, include_archived: bool) -> Vec<&TaskThread> {
        self.threads
            .iter()
            .filter(|thread| thread.project_id == project_id)
            .filter(|thread| include_archived || !thread.archived)
            .collect()
    }

    pub(crate) fn all_threads(&self) -> &[TaskThread] {
        &self.threads
    }

    pub(crate) fn from_loaded_threads(threads: Vec<TaskThread>) -> Self {
        let mut manager = ThreadManager::new();
        manager.project_ids = threads.iter().map(|thread| thread.project_id.clone()).fold(
            Vec::new(),
            |mut ids, id| {
                if !ids.contains(&id) {
                    ids.push(id);
                }
                ids
            },
        );
        manager.next_id = threads
            .iter()
            .filter_map(|thread| thread.id.rsplit("-thread-").next()?.parse::<usize>().ok())
            .max()
            .unwrap_or(threads.len());
        manager.threads = threads;
        manager
    }

    pub fn get_thread(&self, thread_id: &str) -> Result<&TaskThread, ThreadError> {
        self.threads
            .iter()
            .find(|thread| thread.id == thread_id)
            .ok_or(ThreadError::ThreadNotFound)
    }

    pub fn append_message(
        &mut self,
        thread_id: &str,
        role: MessageRole,
        body: &str,
    ) -> Result<ThreadMessage, ThreadError> {
        let thread = self.thread_mut(thread_id)?;
        if thread.archived {
            return Err(ThreadError::ArchivedThread);
        }

        let message = ThreadMessage {
            role,
            body: body.to_string(),
        };
        thread.messages.push(message.clone());
        Ok(message)
    }

    pub fn set_status(&mut self, thread_id: &str, status: ThreadStatus) -> Result<(), ThreadError> {
        let thread = self.thread_mut(thread_id)?;
        if thread.archived {
            return Err(ThreadError::ArchivedThread);
        }
        if !can_transition(thread.status, status) {
            return Err(ThreadError::InvalidTransition);
        }

        thread.status = status;
        Ok(())
    }

    pub fn archive_thread(&mut self, thread_id: &str) -> Result<(), ThreadError> {
        let thread = self.thread_mut(thread_id)?;
        thread.archived = true;
        Ok(())
    }

    fn thread_mut(&mut self, thread_id: &str) -> Result<&mut TaskThread, ThreadError> {
        self.threads
            .iter_mut()
            .find(|thread| thread.id == thread_id)
            .ok_or(ThreadError::ThreadNotFound)
    }
}

fn can_transition(from: ThreadStatus, to: ThreadStatus) -> bool {
    use ThreadStatus::{
        Blocked, Building, Done, Exploring, Failed, Idle, Planning, Reviewing, Testing,
        WaitingForApproval,
    };

    match (from, to) {
        (_, _) if from == to => true,
        (Idle, Exploring | Planning | Blocked | Failed | Done) => true,
        (Exploring, Idle | Planning | Blocked | Failed | Done) => true,
        (Planning, Idle | WaitingForApproval | Building | Blocked | Failed | Done) => true,
        (WaitingForApproval, Building | Blocked | Failed) => true,
        (Building, Testing | Reviewing | Blocked | Failed | Done) => true,
        (Testing, Reviewing | Blocked | Failed | Done) => true,
        (Reviewing, Building | Blocked | Failed | Done) => true,
        (Blocked, Exploring | Planning | Building | Failed | Done) => true,
        (Failed | Done, _) => false,
        _ => false,
    }
}

fn title_from_goal(goal: &str) -> String {
    let title: String = goal.chars().take(48).collect();
    if goal.chars().count() > 48 {
        format!("{title}...")
    } else {
        title
    }
}
