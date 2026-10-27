namespace Sentinel.Agent.Tests;

public class LinkClientReconnectTest
{
    [Fact]
    public async Task RunWithReconnectAsync_delays_double_with_each_failure()
    {
        var delays = new List<TimeSpan>();
        var attempts = 0;
        using var cts = new CancellationTokenSource();
        var client = new LinkClient("127.0.0.1", 0, new SensorPoller(new FakeSensorSource()));

        await Assert.ThrowsAnyAsync<OperationCanceledException>(() =>
            client.RunWithReconnectAsync(
                cts.Token,
                delayFunc: (d, ct) => { delays.Add(d); return Task.CompletedTask; },
                connectFunc: ct =>
                {
                    ct.ThrowIfCancellationRequested();
                    if (++attempts >= 4) cts.Cancel();
                    return Task.FromException(new IOException("disconnected"));
                }));

        Assert.Equal(TimeSpan.FromSeconds(1), delays[0]);
        Assert.Equal(TimeSpan.FromSeconds(2), delays[1]);
        Assert.Equal(TimeSpan.FromSeconds(4), delays[2]);
    }

    [Fact]
    public async Task RunWithReconnectAsync_backoff_caps_at_thirty_seconds()
    {
        var delays = new List<TimeSpan>();
        var attempts = 0;
        using var cts = new CancellationTokenSource();
        var client = new LinkClient("127.0.0.1", 0, new SensorPoller(new FakeSensorSource()));

        await Assert.ThrowsAnyAsync<OperationCanceledException>(() =>
            client.RunWithReconnectAsync(
                cts.Token,
                delayFunc: (d, ct) => { delays.Add(d); return Task.CompletedTask; },
                connectFunc: ct =>
                {
                    ct.ThrowIfCancellationRequested();
                    if (++attempts >= 10) cts.Cancel();
                    return Task.FromException(new IOException("disconnected"));
                }));

        Assert.All(delays, d => Assert.True(d <= TimeSpan.FromSeconds(30)));
        Assert.Contains(TimeSpan.FromSeconds(30), delays);
    }

    [Fact]
    public async Task RunWithReconnectAsync_cancellation_during_delay_stops_loop()
    {
        using var cts = new CancellationTokenSource();
        var client = new LinkClient("127.0.0.1", 0, new SensorPoller(new FakeSensorSource()));

        await Assert.ThrowsAnyAsync<OperationCanceledException>(() =>
            client.RunWithReconnectAsync(
                cts.Token,
                delayFunc: async (d, ct) =>
                {
                    cts.Cancel();
                    await Task.Delay(d, ct);
                },
                connectFunc: ct =>
                {
                    ct.ThrowIfCancellationRequested();
                    return Task.FromException(new IOException("disconnected"));
                }));
    }
}
