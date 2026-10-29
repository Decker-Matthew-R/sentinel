#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MetricState {
    Normal,
    Warm,
    Hot,
}

pub fn cpu_temp_state(temp: f32) -> MetricState {
    if temp >= 85.0 {
        MetricState::Hot
    } else if temp >= 70.0 {
        MetricState::Warm
    } else {
        MetricState::Normal
    }
}

pub fn gpu_temp_state(temp: f32) -> MetricState {
    if temp >= 90.0 {
        MetricState::Hot
    } else if temp >= 80.0 {
        MetricState::Warm
    } else {
        MetricState::Normal
    }
}

pub fn cpu_util_state(util: f32) -> MetricState {
    if util >= 90.0 {
        MetricState::Hot
    } else if util >= 70.0 {
        MetricState::Warm
    } else {
        MetricState::Normal
    }
}

pub fn gpu_util_state(util: f32) -> MetricState {
    if util >= 90.0 {
        MetricState::Hot
    } else if util >= 70.0 {
        MetricState::Warm
    } else {
        MetricState::Normal
    }
}

pub fn gpu_hotspot_state(temp: f32) -> MetricState {
    if temp >= 100.0 {
        MetricState::Hot
    } else if temp >= 90.0 {
        MetricState::Warm
    } else {
        MetricState::Normal
    }
}

pub fn ram_usage_state(used_mb: u64, total_mb: u64) -> MetricState {
    if total_mb == 0 {
        return MetricState::Normal;
    }
    let pct = used_mb as f32 / total_mb as f32 * 100.0;
    if pct >= 90.0 {
        MetricState::Hot
    } else if pct >= 75.0 {
        MetricState::Warm
    } else {
        MetricState::Normal
    }
}

pub fn vram_usage_state(used_mb: u64, total_mb: u64) -> MetricState {
    if total_mb == 0 {
        return MetricState::Normal;
    }
    let pct = used_mb as f32 / total_mb as f32 * 100.0;
    if pct >= 90.0 {
        MetricState::Hot
    } else if pct >= 80.0 {
        MetricState::Warm
    } else {
        MetricState::Normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_temp_normal_below_threshold() {
        assert_eq!(cpu_temp_state(69.9), MetricState::Normal);
    }

    #[test]
    fn cpu_temp_warm_at_threshold() {
        assert_eq!(cpu_temp_state(70.0), MetricState::Warm);
    }

    #[test]
    fn cpu_temp_hot_at_threshold() {
        assert_eq!(cpu_temp_state(85.0), MetricState::Hot);
    }

    #[test]
    fn cpu_util_normal_below_threshold() {
        assert_eq!(cpu_util_state(69.9), MetricState::Normal);
    }

    #[test]
    fn cpu_util_warm_at_threshold() {
        assert_eq!(cpu_util_state(70.0), MetricState::Warm);
    }

    #[test]
    fn cpu_util_hot_at_threshold() {
        assert_eq!(cpu_util_state(90.0), MetricState::Hot);
    }

    #[test]
    fn gpu_util_normal_below_threshold() {
        assert_eq!(gpu_util_state(69.9), MetricState::Normal);
    }

    #[test]
    fn gpu_util_warm_at_threshold() {
        assert_eq!(gpu_util_state(70.0), MetricState::Warm);
    }

    #[test]
    fn gpu_util_hot_at_threshold() {
        assert_eq!(gpu_util_state(90.0), MetricState::Hot);
    }

    #[test]
    fn cpu_temp_warm_just_below_hot() {
        assert_eq!(cpu_temp_state(84.9), MetricState::Warm);
    }

    #[test]
    fn cpu_util_warm_just_below_hot() {
        assert_eq!(cpu_util_state(89.9), MetricState::Warm);
    }

    #[test]
    fn gpu_util_warm_just_below_hot() {
        assert_eq!(gpu_util_state(89.9), MetricState::Warm);
    }

    #[test]
    fn gpu_temp_normal_below_threshold() {
        assert_eq!(gpu_temp_state(79.9), MetricState::Normal);
    }

    #[test]
    fn gpu_temp_warm_at_threshold() {
        assert_eq!(gpu_temp_state(80.0), MetricState::Warm);
    }

    #[test]
    fn gpu_temp_hot_at_threshold() {
        assert_eq!(gpu_temp_state(90.0), MetricState::Hot);
    }

    #[test]
    fn gpu_temp_warm_just_below_hot() {
        assert_eq!(gpu_temp_state(89.9), MetricState::Warm);
    }

    #[test]
    fn gpu_hotspot_normal_below_threshold() {
        assert_eq!(gpu_hotspot_state(89.9), MetricState::Normal);
    }

    #[test]
    fn gpu_hotspot_warm_at_threshold() {
        assert_eq!(gpu_hotspot_state(90.0), MetricState::Warm);
    }

    #[test]
    fn gpu_hotspot_hot_at_threshold() {
        assert_eq!(gpu_hotspot_state(100.0), MetricState::Hot);
    }

    #[test]
    fn gpu_hotspot_warm_just_below_hot() {
        assert_eq!(gpu_hotspot_state(99.9), MetricState::Warm);
    }

    #[test]
    fn ram_usage_normal_below_threshold() {
        assert_eq!(ram_usage_state(7499, 10000), MetricState::Normal);
    }

    #[test]
    fn ram_usage_warm_at_threshold() {
        assert_eq!(ram_usage_state(7500, 10000), MetricState::Warm);
    }

    #[test]
    fn ram_usage_hot_at_threshold() {
        assert_eq!(ram_usage_state(9000, 10000), MetricState::Hot);
    }

    #[test]
    fn ram_usage_zero_total_is_normal() {
        assert_eq!(ram_usage_state(0, 0), MetricState::Normal);
    }

    #[test]
    fn ram_usage_warm_just_below_hot() {
        assert_eq!(ram_usage_state(8999, 10000), MetricState::Warm);
    }

    #[test]
    fn vram_usage_normal_below_threshold() {
        assert_eq!(vram_usage_state(7999, 10000), MetricState::Normal);
    }

    #[test]
    fn vram_usage_warm_at_threshold() {
        assert_eq!(vram_usage_state(8000, 10000), MetricState::Warm);
    }

    #[test]
    fn vram_usage_hot_at_threshold() {
        assert_eq!(vram_usage_state(9000, 10000), MetricState::Hot);
    }

    #[test]
    fn vram_usage_zero_total_is_normal() {
        assert_eq!(vram_usage_state(0, 0), MetricState::Normal);
    }

    #[test]
    fn vram_usage_warm_just_below_hot() {
        assert_eq!(vram_usage_state(8999, 10000), MetricState::Warm);
    }
}
