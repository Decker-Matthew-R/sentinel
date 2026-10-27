namespace Sentinel.Agent;

public record CpuReading(float Utilization, float Temperature);

public record GpuReading(
    float Utilization,
    float Temperature,
    float HotspotTemperature,
    ulong VramUsedMb,
    ulong VramTotalMb);

public record RamReading(ulong UsedMb, ulong TotalMb);

public record SensorReading(
    ulong UptimeSeconds,
    CpuReading? Cpu,
    GpuReading? Gpu,
    RamReading? Ram);
