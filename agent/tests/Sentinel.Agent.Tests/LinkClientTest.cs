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
            var client = new LinkClient("127.0.0.1", port, new SensorPoller(new FakeSensorSource()), fakeSpecs, new FakeActionRunner());
            return client.RunAsync(cts.Token);
        });

        using var server = await listener.AcceptTcpClientAsync();
        var stream = server.GetStream();

        var hello = await FrameReader.ReadAsync(stream);
        Assert.Equal(Envelope.PayloadOneofCase.Hello, hello.PayloadCase);
        Assert.Equal("0.2.0", hello.Hello.AgentVersion);

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

    [Fact]
    public async Task LinkClient_sends_GameInfo_when_steam_library_provided()
    {
        var listener = new TcpListener(IPAddress.Loopback, 0);
        listener.Start();
        var port = ((IPEndPoint)listener.LocalEndpoint).Port;

        using var cts = new CancellationTokenSource();

        var clientTask = Task.Run(() =>
        {
            var client = new LinkClient("127.0.0.1", port, new SensorPoller(new FakeSensorSource()), new FakeSpecsSource(), new FakeActionRunner(), new FakeSteamLibrary());
            return client.RunAsync(cts.Token);
        });

        using var server = await listener.AcceptTcpClientAsync();
        var stream = server.GetStream();

        await FrameReader.ReadAsync(stream); // Hello
        await FrameReader.ReadAsync(stream); // Specs

        var gameInfoEnvelope = await FrameReader.ReadAsync(stream);
        Assert.Equal(Envelope.PayloadOneofCase.GameInfo, gameInfoEnvelope.PayloadCase);
        Assert.Equal("570", gameInfoEnvelope.GameInfo.AppId);
        Assert.Equal("Dota 2", gameInfoEnvelope.GameInfo.Name);

        await cts.CancelAsync();
        await Assert.ThrowsAnyAsync<OperationCanceledException>(() => clientTask);

        listener.Stop();
    }

    [Fact]
    public async Task LinkClient_responds_to_Command_with_CommandResult()
    {
        var listener = new TcpListener(IPAddress.Loopback, 0);
        listener.Start();
        var port = ((IPEndPoint)listener.LocalEndpoint).Port;

        using var cts = new CancellationTokenSource();

        var clientTask = Task.Run(() =>
        {
            var client = new LinkClient("127.0.0.1", port, new SensorPoller(new FakeSensorSource()), new FakeSpecsSource(), new FakeActionRunner());
            return client.RunAsync(cts.Token);
        });

        using var server = await listener.AcceptTcpClientAsync();
        var stream = server.GetStream();

        await FrameReader.ReadAsync(stream); // Hello
        await FrameReader.ReadAsync(stream); // Specs

        await FrameWriter.WriteAsync(stream, new Envelope
        {
            Version = 1,
            Command = new Command { RequestId = "req-1", ActionId = "open-notepad" }
        });

        var response = await FrameReader.ReadAsync(stream);
        Assert.Equal(Envelope.PayloadOneofCase.CommandResult, response.PayloadCase);
        Assert.Equal("req-1", response.CommandResult.RequestId);
        Assert.True(response.CommandResult.Success);

        await cts.CancelAsync();
        await Assert.ThrowsAnyAsync<OperationCanceledException>(() => clientTask);

        listener.Stop();
    }
}