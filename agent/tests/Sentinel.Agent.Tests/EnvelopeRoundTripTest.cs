using Google.Protobuf;
using Sentinel.V1;

namespace Sentinel.Agent.Tests;

public class EnvelopeRoundTripTest
{
    [Fact]
    public void MetricsSnapshot_survives_round_trip()
    {
        var original = new Envelope
        {
            Version = 1,
            MetricsSnapshot = new MetricsSnapshot { UptimeSeconds = 3600 }
        };

        var bytes = original.ToByteArray();
        var decoded = Envelope.Parser.ParseFrom(bytes);

        Assert.Equal(1u, decoded.Version);
        Assert.Equal(Envelope.PayloadOneofCase.MetricsSnapshot, decoded.PayloadCase);
        Assert.Equal(3600UL, decoded.MetricsSnapshot.UptimeSeconds);
    }

    [Fact]
    public void Heartbeat_survives_round_trip()
    {
        var original = new Envelope
        {
            Version = 1,
            Heartbeat = new Heartbeat()
        };

        var bytes = original.ToByteArray();
        var decoded = Envelope.Parser.ParseFrom(bytes);

        Assert.Equal(1u, decoded.Version);
        Assert.Equal(Envelope.PayloadOneofCase.Heartbeat, decoded.PayloadCase);
    }
}