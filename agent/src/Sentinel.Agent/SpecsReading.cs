namespace Sentinel.Agent;

public record SpecsReading(
    string CpuName,
    uint CpuCoreCount,
    uint CpuThreadCount,
    uint CpuBoostClockMhz,
    string GpuName,
    ulong GpuVramTotalMb,
    ulong RamTotalMb,
    uint RamSpeedMhz,
    string RamName,
    string OsVersion);
