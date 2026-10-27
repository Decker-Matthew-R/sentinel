using System.Net.Sockets;
using Sentinel.V1;

namespace Sentinel.Agent;

public class LinkClient(string host, int port, SensorPoller poller, ISpecsSource specsSource)
{
    public async Task RunAsync(CancellationToken cancellationToken)
    {
        using var tcp = new TcpClient();
        await tcp.ConnectAsync(host, port, cancellationToken);

        var stream = tcp.GetStream();

        await FrameWriter.WriteAsync(stream, new Envelope
        {
            Version = 1,
            Hello = new Hello { AgentVersion = "0.1.0" }
        });

        var specs = specsSource.Read();
        await FrameWriter.WriteAsync(stream, new Envelope
        {
            Version = 1,
            Specs = new Specs
            {
                CpuName = specs.CpuName,
                GpuName = specs.GpuName,
                RamTotalMb = specs.RamTotalMb,
                OsVersion = specs.OsVersion
            }
        });

        using var timer = new PeriodicTimer(TimeSpan.FromSeconds(1));
        while (await timer.WaitForNextTickAsync(cancellationToken))
        {
            await FrameWriter.WriteAsync(stream, new Envelope
            {
                Version = 1,
                MetricsSnapshot = poller.PollOnce()
            });
        }
    }

    public async Task RunWithReconnectAsync(
        CancellationToken cancellationToken,
        Func<TimeSpan, CancellationToken, Task>? delayFunc = null,
        Func<CancellationToken, Task>? connectFunc = null)
    {
        delayFunc ??= Task.Delay;
        connectFunc ??= RunAsync;

        var backoff = TimeSpan.FromSeconds(1);
        var maxBackoff = TimeSpan.FromSeconds(30);

        while (true)
        {
            cancellationToken.ThrowIfCancellationRequested();
            try
            {
                await connectFunc(cancellationToken);
                return;
            }
            catch (OperationCanceledException)
            {
                throw;
            }
            catch (Exception)
            {
            }

            await delayFunc(backoff, cancellationToken);
            backoff = TimeSpan.FromTicks(Math.Min(backoff.Ticks * 2, maxBackoff.Ticks));
        }
    }
}