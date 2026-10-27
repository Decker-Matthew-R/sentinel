use crate::proto::sentinel::v1::MetricsSnapshot;
use tokio::sync::watch;

pub struct StateStore {
    tx: watch::Sender<Option<MetricsSnapshot>>,
}

impl StateStore {
    pub fn new() -> (Self, watch::Receiver<Option<MetricsSnapshot>>) {
        let (tx, rx) = watch::channel(None);
        (Self { tx }, rx)
    }

    pub fn update(&self, snapshot: MetricsSnapshot) {
        self.tx.send(Some(snapshot)).ok();
    }

    pub fn clear(&self) {
        self.tx.send(None).ok();
    }

    #[allow(dead_code)]
    pub fn current(&self) -> Option<MetricsSnapshot> {
        *self.tx.borrow()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::sentinel::v1::MetricsSnapshot;

    fn snapshot(uptime: u64) -> MetricsSnapshot {
        MetricsSnapshot {
            uptime_seconds: uptime,
            ..Default::default()
        }
    }

    #[test]
    fn initial_state_is_none() {
        let (store, _rx) = StateStore::new();
        assert!(store.current().is_none());
    }

    #[test]
    fn update_stores_snapshot() {
        let (store, _rx) = StateStore::new();
        store.update(snapshot(120));
        assert_eq!(store.current().unwrap().uptime_seconds, 120);
    }

    #[test]
    fn update_replaces_previous_snapshot() {
        let (store, _rx) = StateStore::new();
        store.update(snapshot(10));
        store.update(snapshot(20));
        assert_eq!(store.current().unwrap().uptime_seconds, 20);
    }

    #[test]
    fn clear_resets_to_none() {
        let (store, _rx) = StateStore::new();
        store.update(snapshot(60));
        store.clear();
        assert!(store.current().is_none());
    }
}
