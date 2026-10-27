using System.Net;
using System.Net.Sockets;
using Sentinel.V1;

namespace Sentinel.Agent.Tests;

public class LinkClientTest
{
    [Fact]
    public async Task LinkClient_sends_Hello_then_MetricsSnapshot()
    {
        var listener = new TcpListener(IPAddress.Loopback, 0);
        listener.Start();
        var port = ((IPEndPoint)listener.LocalEndpoint).Port;

        using var cts = new CancellationTokenSource();

        var fakeSpecs = new FakeSpecsSource();
        var clientTask = Task.Run(() =>
        {
            var client = new LinkClient("127.0.0.1", port, new SensorPoller(new FakeSensorSource()), fakeSpecs);
            return client.RunAsync(cts.Token);
        });

        using var server = await listener.AcceptTcpClientAsync();
        var stream = server.GetStream();

        var hello = await FrameReader.ReadAsync(stream);
        Assert.Equal(Envelope.PayloadOneofCase.Hello, hello.PayloadCase);
        Assert.Equal("0.1.0", hello.Hello.AgentVersion);

        var specsEnvelope = await FrameReader.ReadAsync(stream);
        Assert.Equal(Envelope.PayloadOneofCase.Specs, specsEnvelope.PayloadCase);
        var expected = fakeSpecs.Read();
        Assert.Equal(expected.CpuName, specsEnvelope.Specs.CpuName);
        Assert.Equal(expected.GpuName, specsEnvelope.Specs.GpuName);
        Assert.Equal(expected.RamTotalMb, specsEnvelope.Specs.RamTotalMb);
        Assert.Equal(expected.OsVersion, specsEnvelope.Specs.OsVersion);

        var metrics = await FrameReader.ReadAsync(stream);
        Assert.Equal(Envelope.PayloadOneofCase.MetricsSnapshot, metrics.PayloadCase);
        Assert.True(metrics.MetricsSnapshot.UptimeSeconds > 0);

        await cts.CancelAsync();
        await Assert.ThrowsAnyAsync<OperationCanceledException>(() => clientTask);

        listener.Stop();
    }
}