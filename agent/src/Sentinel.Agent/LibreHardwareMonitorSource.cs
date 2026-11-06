using LibreHardwareMonitor.Hardware;

namespace Sentinel.Agent;

public class LibreHardwareMonitorSource(Computer computer) : ISensorSource
{
    public SensorReading Read()
    {
        foreach (var hardware in computer.Hardware)
            hardware.Update();

        return new SensorReading(
            UptimeSeconds: (ulong)(Environment.TickCount64 / 1000),
            Cpu: ReadCpu(),
            Gpu: ReadGpu(),
            Ram: ReadRam()
        );
    }

    private CpuReading? ReadCpu()
    {
        var hw = computer.Hardware.FirstOrDefault(h => h.HardwareType == HardwareType.Cpu);
        if (hw is null) return null;

        var utilization = SensorValue(hw, SensorType.Load, "CPU Total");
        var temp = SensorValue(hw, SensorType.Temperature, "CPU Package")
            ?? hw.Sensors.FirstOrDefault(s => s.SensorType == SensorType.Temperature)?.Value;

        if (utilization is null && temp is null) return null;
        return new CpuReading(Utilization: utilization ?? 0f, Temperature: temp ?? 0f);
    }

    private GpuReading? ReadGpu()
    {
        var hw = computer.Hardware.FirstOrDefault(h =>
            h.HardwareType is HardwareType.GpuNvidia or HardwareType.GpuAmd or HardwareType.GpuIntel);
        if (hw is null) return null;

        var utilization = SensorValue(hw, SensorType.Load, "GPU Core");
        var temp = SensorValue(hw, SensorType.Temperature, "GPU Core");
        var hotspot = SensorValue(hw, SensorType.Temperature, "GPU Hot Spot");
        var vramUsed = SensorValue(hw, SensorType.SmallData, "GPU Memory Used");
        var vramTotal = SensorValue(hw, SensorType.SmallData, "GPU Memory Total");

        if (utilization is null && temp is null) return null;
        return new GpuReading(
            Utilization: utilization ?? 0f,
            Temperature: temp ?? 0f,
            HotspotTemperature: hotspot ?? temp ?? 0f,
            VramUsedMb: (ulong)(vramUsed ?? 0f),
            VramTotalMb: (ulong)(vramTotal ?? 0f)
        );
    }

    private RamReading? ReadRam()
    {
        var hw = computer.Hardware.FirstOrDefault(h => h.HardwareType == HardwareType.Memory);
        if (hw is null) return null;

        var usedGb = SensorValue(hw, SensorType.Data, "Memory Used");
        var availableGb = SensorValue(hw, SensorType.Data, "Memory Available");

        if (usedGb is null) return null;

        var usedMb = (ulong)(usedGb.Value * 1024);
        var totalMb = availableGb.HasValue ? usedMb + (ulong)(availableGb.Value * 1024) : 0ul;
        return new RamReading(UsedMb: usedMb, TotalMb: totalMb);
    }

    private static float? SensorValue(IHardware hardware, SensorType type, string name) =>
        hardware.Sensors.FirstOrDefault(s => s.SensorType == type && s.Name == name)?.Value;
}
