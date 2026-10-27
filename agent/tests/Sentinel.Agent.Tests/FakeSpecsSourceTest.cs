namespace Sentinel.Agent.Tests;

public class FakeSpecsSourceTest
{
    [Fact]
    public void Read_returns_non_empty_cpu_name()
    {
        var reading = new FakeSpecsSource().Read();
        Assert.False(string.IsNullOrEmpty(reading.CpuName));
    }

    [Fact]
    public void Read_returns_non_empty_gpu_name()
    {
        var reading = new FakeSpecsSource().Read();
        Assert.False(string.IsNullOrEmpty(reading.GpuName));
    }

    [Fact]
    public void Read_returns_positive_ram_total()
    {
        var reading = new FakeSpecsSource().Read();
        Assert.True(reading.RamTotalMb > 0);
    }

    [Fact]
    public void Read_returns_non_empty_os_version()
    {
        var reading = new FakeSpecsSource().Read();
        Assert.False(string.IsNullOrEmpty(reading.OsVersion));
    }
}
