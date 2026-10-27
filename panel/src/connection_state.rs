#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Waiting,
    Connected {
        agent_version: String,
        uptime_seconds: u64,
    },
    Stale {
        agent_version: String,
    },
}
