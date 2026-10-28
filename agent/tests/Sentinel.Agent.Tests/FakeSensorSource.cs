namespace Sentinel.Agent;

public class FakeSensorSource : ISensorSource
{
    private ulong _uptimeSeconds;
    private double _phase;

    public SensorReading Read()
    {
        _uptimeSeconds++;
        _phase += 0.05;

        var cpuUtil = OscillatingValue(_phase, min: 5f, max: 85f);
        var gpuUtil = OscillatingValue(_phase + 1.0, min: 10f, max: 95f);

        return new SensorReading(
            UptimeSeconds: _uptimeSeconds,
            Cpu: new CpuReading(
                Utilization: cpuUtil,
                Temperature: 35f + cpuUtil * 0.55f),
            Gpu: new GpuReading(
                Utilization: gpuUtil,
                Temperature: 40f + gpuUtil * 0.50f,
                HotspotTemperature: 50f + gpuUtil * 0.60f,
                VramUsedMb: 4096 + (ulong)(gpuUtil * 20),
                VramTotalMb: 12288),
            Ram: new RamReading(UsedMb: 14336, TotalMb: 32768));
    }

    private static float OscillatingValue(double phase, float min, float max) =>
        min + (max - min) * (float)((Math.Sin(phase) + 1.0) / 2.0);
}
