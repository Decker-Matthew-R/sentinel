use crate::proto::sentinel::v1::MetricsSnapshot;
use std::collections::VecDeque;
use std::sync::Mutex;
use tokio::sync::watch;

const HISTORY_CAPACITY: usize = 60;

pub struct MetricHistory {
    values: VecDeque<f32>,
    capacity: usize,
}

#[allow(dead_code)]
impl MetricHistory {
    pub fn new(capacity: usize) -> Self {
        Self {
            values: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, value: f32) {
        if self.values.len() == self.capacity {
            self.values.pop_front();
        }
        self.values.push_back(value);
    }

    pub fn values(&self) -> &VecDeque<f32> {
        &self.values
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

pub struct MetricHistories {
    pub cpu_utilization: MetricHistory,
    pub cpu_temperature: MetricHistory,
    pub gpu_utilization: MetricHistory,
    pub gpu_temperature: MetricHistory,
    pub gpu_hotspot_temperature: MetricHistory,
    pub gpu_vram_used_mb: MetricHistory,
    pub ram_used_mb: MetricHistory,
}

impl MetricHistories {
    fn new(capacity: usize) -> Self {
        Self {
            cpu_utilization: MetricHistory::new(capacity),
            cpu_temperature: MetricHistory::new(capacity),
            gpu_utilization: MetricHistory::new(capacity),
            gpu_temperature: MetricHistory::new(capacity),
            gpu_hotspot_temperature: MetricHistory::new(capacity),
            gpu_vram_used_mb: MetricHistory::new(capacity),
            ram_used_mb: MetricHistory::new(capacity),
        }
    }

    fn push(&mut self, snapshot: &MetricsSnapshot) {
        if let Some(cpu) = &snapshot.cpu {
            self.cpu_utilization.push(cpu.utilization);
            self.cpu_temperature.push(cpu.temperature);
        }
        if let Some(gpu) = &snapshot.gpu {
            self.gpu_utilization.push(gpu.utilization);
            self.gpu_temperature.push(gpu.temperature);
            self.gpu_hotspot_temperature.push(gpu.hotspot_temperature);
            self.gpu_vram_used_mb.push(gpu.vram_used_mb as f32);
        }
        if let Some(ram) = &snapshot.ram {
            self.ram_used_mb.push(ram.used_mb as f32);
        }
    }

    fn clear(&mut self) {
        self.cpu_utilization.values.clear();
        self.cpu_temperature.values.clear();
        self.gpu_utilization.values.clear();
        self.gpu_temperature.values.clear();
        self.gpu_hotspot_temperature.values.clear();
        self.gpu_vram_used_mb.values.clear();
        self.ram_used_mb.values.clear();
    }
}

pub struct StateStore {
    tx: watch::Sender<Option<MetricsSnapshot>>,
    histories: Mutex<MetricHistories>,
}

impl StateStore {
    pub fn new() -> (Self, watch::Receiver<Option<MetricsSnapshot>>) {
        let (tx, rx) = watch::channel(None);
        let store = Self {
            tx,
            histories: Mutex::new(MetricHistories::new(HISTORY_CAPACITY)),
        };
        (store, rx)
    }

    pub fn update(&self, snapshot: MetricsSnapshot) {
        self.histories.lock().unwrap().push(&snapshot);
        self.tx.send(Some(snapshot)).ok();
    }

    pub fn clear(&self) {
        self.histories.lock().unwrap().clear();
        self.tx.send(None).ok();
    }

    #[allow(dead_code)]
    pub fn current(&self) -> Option<MetricsSnapshot> {
        *self.tx.borrow()
    }

    #[allow(dead_code)]
    pub fn histories(&self) -> std::sync::MutexGuard<'_, MetricHistories> {
        self.histories.lock().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::sentinel::v1::{CpuMetrics, GpuMetrics, RamMetrics};

    fn snapshot(uptime: u64) -> MetricsSnapshot {
        MetricsSnapshot {
            uptime_seconds: uptime,
            ..Default::default()
        }
    }

    fn full_snapshot(cpu_util: f32, gpu_util: f32, ram_used: u64) -> MetricsSnapshot {
        MetricsSnapshot {
            uptime_seconds: 1,
            cpu: Some(CpuMetrics {
                utilization: cpu_util,
                temperature: 50.0,
            }),
            gpu: Some(GpuMetrics {
                utilization: gpu_util,
                temperature: 60.0,
                hotspot_temperature: 70.0,
                vram_used_mb: 4096,
                vram_total_mb: 12288,
            }),
            ram: Some(RamMetrics {
                used_mb: ram_used,
                total_mb: 32768,
            }),
        }
    }

    #[test]
    fn push_adds_values() {
        let mut h = MetricHistory::new(10);
        h.push(1.0);
        h.push(2.0);
        assert_eq!(h.len(), 2);
    }

    #[test]
    fn push_drops_oldest_when_full() {
        let mut h = MetricHistory::new(3);
        h.push(1.0);
        h.push(2.0);
        h.push(3.0);
        h.push(4.0);
        assert_eq!(h.len(), 3);
        assert_eq!(*h.values().front().unwrap(), 2.0);
        assert_eq!(*h.values().back().unwrap(), 4.0);
    }

    #[test]
    fn values_are_in_insertion_order() {
        let mut h = MetricHistory::new(5);
        for i in 1..=5 {
            h.push(i as f32);
        }
        let vals: Vec<f32> = h.values().iter().copied().collect();
        assert_eq!(vals, vec![1.0, 2.0, 3.0, 4.0, 5.0]);
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

    #[test]
    fn histories_start_empty() {
        let (store, _rx) = StateStore::new();
        let h = store.histories();
        assert!(h.cpu_utilization.is_empty());
        assert!(h.gpu_utilization.is_empty());
        assert!(h.ram_used_mb.is_empty());
    }

    #[test]
    fn histories_grow_on_update() {
        let (store, _rx) = StateStore::new();
        store.update(full_snapshot(50.0, 80.0, 8192));
        store.update(full_snapshot(60.0, 90.0, 8192));
        let h = store.histories();
        assert_eq!(h.cpu_utilization.len(), 2);
        assert_eq!(h.cpu_temperature.len(), 2);
        assert_eq!(h.gpu_utilization.len(), 2);
        assert_eq!(h.gpu_temperature.len(), 2);
        assert_eq!(h.gpu_hotspot_temperature.len(), 2);
        assert_eq!(h.gpu_vram_used_mb.len(), 2);
        assert_eq!(h.ram_used_mb.len(), 2);
    }

    #[test]
    fn histories_cap_at_capacity() {
        let (store, _rx) = StateStore::new();
        for i in 0..70 {
            store.update(full_snapshot(i as f32, i as f32, i as u64));
        }
        let h = store.histories();
        assert_eq!(h.cpu_utilization.len(), HISTORY_CAPACITY);
    }

    #[test]
    fn histories_cleared_on_clear() {
        let (store, _rx) = StateStore::new();
        store.update(full_snapshot(50.0, 80.0, 8192));
        store.clear();
        let h = store.histories();
        assert!(h.cpu_utilization.is_empty());
        assert!(h.cpu_temperature.is_empty());
        assert!(h.gpu_utilization.is_empty());
        assert!(h.gpu_temperature.is_empty());
        assert!(h.gpu_hotspot_temperature.is_empty());
        assert!(h.gpu_vram_used_mb.is_empty());
        assert!(h.ram_used_mb.is_empty());
    }

    #[test]
    fn absent_sub_message_leaves_history_unchanged() {
        let (store, _rx) = StateStore::new();
        store.update(snapshot(1));
        let h = store.histories();
        assert!(h.cpu_utilization.is_empty());
        assert!(h.gpu_utilization.is_empty());
        assert!(h.ram_used_mb.is_empty());
    }
}
