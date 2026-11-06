using System.Net.Sockets;
using Sentinel.V1;

namespace Sentinel.Agent;

public class LinkClient(string host, int port, SensorPoller poller, ISpecsSource specsSource, IActionRunner actionRunner)
{
    private readonly SemaphoreSlim _writeLock = new(1, 1);

    public async Task RunAsync(CancellationToken cancellationToken)
    {
        using var tcp = new TcpClient();
        await tcp.ConnectAsync(host, port, cancellationToken);

        var stream = tcp.GetStream();

        await SendAsync(stream, new Envelope
        {
            Version = 1,
            Hello = new Hello { AgentVersion = "0.1.0" }
        });

        var specs = specsSource.Read();
        await SendAsync(stream, new Envelope
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

        _ = ReadCommandsAsync(stream, cancellationToken);

        using var timer = new PeriodicTimer(TimeSpan.FromSeconds(1));
        while (await timer.WaitForNextTickAsync(cancellationToken))
        {
            await SendAsync(stream, new Envelope
            {
                Version = 1,
                MetricsSnapshot = poller.PollOnce()
            });
        }
    }

    private async Task ReadCommandsAsync(NetworkStream stream, CancellationToken cancellationToken)
    {
        while (!cancellationToken.IsCancellationRequested)
        {
            Envelope envelope;
            try
            {
                envelope = await FrameReader.ReadAsync(stream);
            }
            catch
            {
                return;
            }

            if (envelope.PayloadCase != Envelope.PayloadOneofCase.Command)
                continue;

            var result = await actionRunner.ExecuteAsync(envelope.Command);
            try
            {
                await SendAsync(stream, new Envelope { Version = 1, CommandResult = result });
            }
            catch
            {
                return;
            }
        }
    }

    private async Task SendAsync(NetworkStream stream, Envelope envelope)
    {
        await _writeLock.WaitAsync();
        try
        {
            await FrameWriter.WriteAsync(stream, envelope);
        }
        finally
        {
            _writeLock.Release();
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
