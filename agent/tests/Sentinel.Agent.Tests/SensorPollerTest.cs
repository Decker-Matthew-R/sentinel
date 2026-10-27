namespace Sentinel.Agent.Tests;

public class SensorPollerTest
{
    private static SensorReading Reading(
        ulong uptime = 0,
        CpuReading? cpu = null,
        GpuReading? gpu = null,
        RamReading? ram = null) =>
        new(uptime, cpu, gpu, ram);

    private static CpuReading Cpu() => new(Utilization: 42.0f, Temperature: 55.0f);

    private static GpuReading Gpu() => new(
        Utilization: 80.0f,
        Temperature: 65.0f,
        HotspotTemperature: 75.0f,
        VramUsedMb: 4096,
        VramTotalMb: 12288);

    private static RamReading Ram() => new(UsedMb: 8192, TotalMb: 32768);

    [Fact]
    public void ToSnapshot_maps_uptime_seconds()
    {
        var snapshot = SensorPoller.ToSnapshot(Reading(uptime: 123));
        Assert.Equal(123UL, snapshot.UptimeSeconds);
    }

    [Fact]
    public void ToSnapshot_maps_cpu_fields()
    {
        var snapshot = SensorPoller.ToSnapshot(Reading(cpu: Cpu()));
        Assert.NotNull(snapshot.Cpu);
        Assert.Equal(42.0f, snapshot.Cpu.Utilization);
        Assert.Equal(55.0f, snapshot.Cpu.Temperature);
    }

    [Fact]
    public void ToSnapshot_maps_gpu_fields()
    {
        var snapshot = SensorPoller.ToSnapshot(Reading(gpu: Gpu()));
        Assert.NotNull(snapshot.Gpu);
        Assert.Equal(80.0f, snapshot.Gpu.Utilization);
        Assert.Equal(65.0f, snapshot.Gpu.Temperature);
        Assert.Equal(75.0f, snapshot.Gpu.HotspotTemperature);
        Assert.Equal(4096UL, snapshot.Gpu.VramUsedMb);
        Assert.Equal(12288UL, snapshot.Gpu.VramTotalMb);
    }

    [Fact]
    public void ToSnapshot_maps_ram_fields()
    {
        var snapshot = SensorPoller.ToSnapshot(Reading(ram: Ram()));
        Assert.NotNull(snapshot.Ram);
        Assert.Equal(8192UL, snapshot.Ram.UsedMb);
        Assert.Equal(32768UL, snapshot.Ram.TotalMb);
    }

    [Fact]
    public void ToSnapshot_null_cpu_omits_cpu_from_snapshot()
    {
        var snapshot = SensorPoller.ToSnapshot(Reading(cpu: null));
        Assert.Null(snapshot.Cpu);
    }

    [Fact]
    public void ToSnapshot_null_gpu_omits_gpu_from_snapshot()
    {
        var snapshot = SensorPoller.ToSnapshot(Reading(gpu: null));
        Assert.Null(snapshot.Gpu);
    }

    [Fact]
    public void ToSnapshot_null_ram_omits_ram_from_snapshot()
    {
        var snapshot = SensorPoller.ToSnapshot(Reading(ram: null));
        Assert.Null(snapshot.Ram);
    }
}
