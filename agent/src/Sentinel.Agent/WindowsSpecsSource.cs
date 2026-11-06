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

        ulong gpuVramTotalMb = 0;
        if (gpu is not null)
        {
            gpu.Update();
            gpuVramTotalMb = (ulong)(gpu.Sensors
                .FirstOrDefault(s => s.SensorType == SensorType.SmallData && s.Name == "GPU Memory Total")
                ?.Value ?? 0f);
        }

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

        var smbios = computer.SMBios;
        var procInfo = smbios.Processors.FirstOrDefault();
        var memDevice = smbios.MemoryDevices.FirstOrDefault(m => m.Size > 0);

        _cached = new SpecsReading(
            CpuName: cpu?.Name ?? "Unknown CPU",
            CpuCoreCount: procInfo?.CoreCount ?? 0,
            CpuThreadCount: procInfo?.ThreadCount ?? 0,
            CpuBoostClockMhz: procInfo?.MaxSpeed ?? 0,
            GpuName: gpu?.Name ?? "Unknown GPU",
            GpuVramTotalMb: gpuVramTotalMb,
            RamTotalMb: ramTotalMb,
            RamSpeedMhz: memDevice?.Speed ?? 0,
            RamName: memDevice?.Type.ToString() ?? "",
            OsVersion: RuntimeInformation.OSDescription
        );
    }

    public SpecsReading Read() => _cached;
}
