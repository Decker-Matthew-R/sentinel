using Google.Protobuf;
using Sentinel.V1;

namespace Sentinel.Agent.Tests;

public class FramingTest
{
    [Fact]
    public async Task FrameWriter_and_FrameReader_round_trip_a_single_frame()
    {
        var original = new Envelope
        {
            Version = 1,
            MetricsSnapshot = new MetricsSnapshot { UptimeSeconds = 9999 }
        };

        var stream = new MemoryStream();
        await FrameWriter.WriteAsync(stream, original);

        stream.Position = 0;
        var decoded = await FrameReader.ReadAsync(stream);

        Assert.Equal(Envelope.PayloadOneofCase.MetricsSnapshot, decoded.PayloadCase);
        Assert.Equal(9999UL, decoded.MetricsSnapshot.UptimeSeconds);
    }

    [Fact]
    public async Task FrameReader_reads_multiple_consecutive_frames()
    {
        var first = new Envelope { Version = 1, Heartbeat = new Heartbeat() };
        var second = new Envelope
        {
            Version = 1,
            MetricsSnapshot = new MetricsSnapshot { UptimeSeconds = 42 }
        };

        var stream = new MemoryStream();
        await FrameWriter.WriteAsync(stream, first);
        await FrameWriter.WriteAsync(stream, second);

        stream.Position = 0;
        var decodedFirst = await FrameReader.ReadAsync(stream);
        var decodedSecond = await FrameReader.ReadAsync(stream);

        Assert.Equal(Envelope.PayloadOneofCase.Heartbeat, decodedFirst.PayloadCase);
        Assert.Equal(42UL, decodedSecond.MetricsSnapshot.UptimeSeconds);
    }

    [Fact]
    public async Task FrameReader_throws_when_stream_closes_during_length()
    {
        var stream = new MemoryStream(new byte[] { 0x00, 0x00 });

        await Assert.ThrowsAsync<EndOfStreamException>(
            () => FrameReader.ReadAsync(stream));
    }

    [Fact]
    public async Task FrameReader_throws_when_stream_closes_during_payload()
    {
        var stream = new MemoryStream();

        var length = BitConverter.GetBytes(100);
        if (BitConverter.IsLittleEndian) Array.Reverse(length);
        await stream.WriteAsync(length);
        await stream.WriteAsync(new byte[10]);

        stream.Position = 0;
        await Assert.ThrowsAsync<EndOfStreamException>(
            () => FrameReader.ReadAsync(stream));
    }
}