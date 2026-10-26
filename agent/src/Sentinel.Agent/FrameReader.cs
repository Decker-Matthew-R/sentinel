using Google.Protobuf;
using Sentinel.V1;

namespace Sentinel.Agent;

public static class FrameReader
{
    public static async Task<Envelope> ReadAsync(Stream stream)
    {
        var lengthBytes = new byte[4];
        await stream.ReadExactlyAsync(lengthBytes);

        if (BitConverter.IsLittleEndian)
            Array.Reverse(lengthBytes);

        var length = BitConverter.ToInt32(lengthBytes);

        var payload = new byte[length];
        await stream.ReadExactlyAsync(payload);

        return Envelope.Parser.ParseFrom(payload);
    }
}