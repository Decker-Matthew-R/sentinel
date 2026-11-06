using LibreHardwareMonitor.Hardware;

namespace Sentinel.Agent.Tests;

public class LibreHardwareMonitorSourceTest
{
    [Fact(Skip = "Requires elevated Windows process with physical hardware — run manually on the target PC")]
    public void ReadReturnsCpuMetrics()
    {
        var computer = OpenComputer();
        var source = new LibreHardwareMonitorSource(computer);

        var reading = source.Read();

        Assert.NotNull(reading.Cpu);
        Assert.InRange(reading.Cpu.Utilization, 0f, 100f);
        Assert.InRange(reading.Cpu.Temperature, 0f, 120f);

        computer.Close();
    }

    [Fact(Skip = "Requires elevated Windows process with physical hardware — run manually on the target PC")]
    public void ReadReturnsGpuMetrics()
    {
        var computer = OpenComputer();
        var source = new LibreHardwareMonitorSource(computer);

        var reading = source.Read();

        Assert.NotNull(reading.Gpu);
        Assert.InRange(reading.Gpu.Utilization, 0f, 100f);
        Assert.InRange(reading.Gpu.Temperature, 0f, 120f);
        Assert.True(reading.Gpu.VramTotalMb > 0);

        computer.Close();
    }

    [Fact(Skip = "Requires elevated Windows process with physical hardware — run manually on the target PC")]
    public void ReadReturnsRamMetrics()
    {
        var computer = OpenComputer();
        var source = new LibreHardwareMonitorSource(computer);

        var reading = source.Read();

        Assert.NotNull(reading.Ram);
        Assert.True(reading.Ram.TotalMb > 0);
        Assert.True(reading.Ram.UsedMb <= reading.Ram.TotalMb);

        computer.Close();
    }

    [Fact(Skip = "Requires elevated Windows process with physical hardware — run manually on the target PC")]
    public void ReadReturnsPositiveUptime()
    {
        var computer = OpenComputer();
        var source = new LibreHardwareMonitorSource(computer);

        var reading = source.Read();

        Assert.True(reading.UptimeSeconds > 0);

        computer.Close();
    }

    private static Computer OpenComputer()
    {
        var computer = new Computer
        {
            IsCpuEnabled = true,
            IsGpuEnabled = true,
            IsMemoryEnabled = true,
        };
        computer.Open();
        return computer;
    }
}
