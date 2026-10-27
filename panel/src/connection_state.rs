#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Waiting,
    Connected { agent_version: String },
    Stale { agent_version: String },
}
