namespace Sentinel.Agent;

using Google.Protobuf;

public static class FrameWriter
{
    public static async Task WriteAsync(Stream stream, IMessage message)
    {
        var payload = message.ToByteArray();
        var length = BitConverter.GetBytes(payload.Length);

        if (BitConverter.IsLittleEndian)
            Array.Reverse(length);

        await stream.WriteAsync(length);
        await stream.WriteAsync(payload);
    }
}