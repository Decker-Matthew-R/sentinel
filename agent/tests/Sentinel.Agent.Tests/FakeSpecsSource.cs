namespace Sentinel.Agent;

public class FakeSpecsSource : ISpecsSource
{
    public SpecsReading Read() =>
        new("Fake CPU", 8, 16, 5000, "Fake GPU", 8192, 32768, 3200, "DDR4", "Fake OS 1.0");
}
