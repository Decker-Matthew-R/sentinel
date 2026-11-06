using System.Runtime.InteropServices;
using LibreHardwareMonitor.Hardware;

namespace Sentinel.Agent;

public class WindowsSpecsSource : ISpecsSource
{
    private readonly SpecsReading _cached;

    public WindowsSpecsSource(Computer computer)
    {
        var cpu = computer.Hardware.FirstOrDefault(h => h.HardwareType == HardwareType.Cpu);
        var gpu = computer.Hardware.FirstOrDefault(h =>
            h.HardwareType is HardwareType.GpuNvidia or HardwareType.GpuAmd or HardwareType.GpuIntel);
        var mem = computer.Hardware.FirstOrDefault(h => h.HardwareType == HardwareType.Memory);

        ulong ramTotalMb = 0;
        if (mem is not null)
        {
            mem.Update();
            var usedGb = mem.Sensors
                .FirstOrDefault(s => s.SensorType == SensorType.Data && s.Name == "Memory Used")?.Value;
            var availableGb = mem.Sensors
                .FirstOrDefault(s => s.SensorType == SensorType.Data && s.Name == "Memory Available")?.Value;
            ramTotalMb = (ulong)((usedGb.GetValueOrDefault() + availableGb.GetValueOrDefault()) * 1024);
        }

        _cached = new SpecsReading(
            CpuName: cpu?.Name ?? "Unknown CPU",
            GpuName: gpu?.Name ?? "Unknown GPU",
            RamTotalMb: ramTotalMb,
            OsVersion: RuntimeInformation.OSDescription
        );
    }

    public SpecsReading Read() => _cached;
}
