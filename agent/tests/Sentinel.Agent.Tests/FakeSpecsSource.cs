namespace Sentinel.Agent;

public class FakeSpecsSource : ISpecsSource
{
    public SpecsReading Read() =>
        new("Fake CPU", "Fake GPU", 32768, "Fake OS 1.0");
}
