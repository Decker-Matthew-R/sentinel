using System.Net.Sockets;
using Sentinel.V1;

namespace Sentinel.Agent;

public class LinkClient(string host, int port)
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

        using var timer = new PeriodicTimer(TimeSpan.FromSeconds(1));
        while (await timer.WaitForNextTickAsync(cancellationToken))
        {
            await FrameWriter.WriteAsync(stream, new Envelope
            {
                Version = 1,
                Heartbeat = new Heartbeat()
            });
        }
    }
}