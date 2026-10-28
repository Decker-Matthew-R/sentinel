namespace Sentinel.Agent;

public record SpecsReading(string CpuName, string GpuName, ulong RamTotalMb, string OsVersion);
