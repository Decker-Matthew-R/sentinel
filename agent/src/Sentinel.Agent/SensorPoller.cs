using Sentinel.V1;

namespace Sentinel.Agent;

public class SensorPoller(ISensorSource source)
{
    public MetricsSnapshot PollOnce() => ToSnapshot(source.Read());

    public static MetricsSnapshot ToSnapshot(SensorReading r)
    {
        var snapshot = new MetricsSnapshot { UptimeSeconds = r.UptimeSeconds };

        if (r.Cpu is { } cpu)
            snapshot.Cpu = new CpuMetrics { Utilization = cpu.Utilization, Temperature = cpu.Temperature };

        if (r.Gpu is { } gpu)
            snapshot.Gpu = new GpuMetrics
            {
                Utilization = gpu.Utilization,
                Temperature = gpu.Temperature,
                HotspotTemperature = gpu.HotspotTemperature,
                VramUsedMb = gpu.VramUsedMb,
                VramTotalMb = gpu.VramTotalMb
            };

        if (r.Ram is { } ram)
            snapshot.Ram = new RamMetrics { UsedMb = ram.UsedMb, TotalMb = ram.TotalMb };

        return snapshot;
    }
}
