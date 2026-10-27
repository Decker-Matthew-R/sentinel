namespace Sentinel.Agent.Tests;

public class FakeSensorSourceTest
{
    [Fact]
    public void Read_returns_cpu_gpu_and_ram_readings()
    {
        var source = new FakeSensorSource();
        var reading = source.Read();

        Assert.NotNull(reading.Cpu);
        Assert.NotNull(reading.Gpu);
        Assert.NotNull(reading.Ram);
    }

    [Fact]
    public void Uptime_increments_on_each_read()
    {
        var source = new FakeSensorSource();
        var first = source.Read().UptimeSeconds;
        var second = source.Read().UptimeSeconds;

        Assert.Equal(first + 1, second);
    }

    [Fact]
    public void Cpu_utilization_is_within_range()
    {
        var source = new FakeSensorSource();
        for (var i = 0; i < 20; i++)
        {
            var cpu = source.Read().Cpu!;
            Assert.InRange(cpu.Utilization, 0f, 100f);
            Assert.InRange(cpu.Temperature, 0f, 120f);
        }
    }

    [Fact]
    public void Gpu_utilization_and_vram_are_within_range()
    {
        var source = new FakeSensorSource();
        for (var i = 0; i < 20; i++)
        {
            var gpu = source.Read().Gpu!;
            Assert.InRange(gpu.Utilization, 0f, 100f);
            Assert.InRange(gpu.Temperature, 0f, 120f);
            Assert.InRange(gpu.HotspotTemperature, 0f, 120f);
            Assert.True(gpu.VramUsedMb <= gpu.VramTotalMb);
        }
    }

    [Fact]
    public void Ram_used_does_not_exceed_total()
    {
        var source = new FakeSensorSource();
        var ram = source.Read().Ram!;
        Assert.True(ram.UsedMb <= ram.TotalMb);
    }
}
