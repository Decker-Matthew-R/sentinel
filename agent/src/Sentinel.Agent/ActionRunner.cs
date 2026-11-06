using System.Diagnostics;
using Sentinel.V1;

namespace Sentinel.Agent;

public class ActionRunner(
    IReadOnlyDictionary<string, string[]> whitelistedActions,
    Func<ProcessStartInfo, Process?>? processLauncher = null) : IActionRunner
{
    private readonly Func<ProcessStartInfo, Process?> _launch = processLauncher ?? Process.Start;

    public Task<CommandResult> ExecuteAsync(Command command)
    {
        if (!whitelistedActions.TryGetValue(command.ActionId, out var argv) || argv.Length == 0)
            return Task.FromResult(new CommandResult
            {
                RequestId = command.RequestId,
                Success = false,
                ErrorMessage = $"Unknown action: {command.ActionId}"
            });

        var psi = new ProcessStartInfo
        {
            FileName = argv[0],
            UseShellExecute = false
        };
        for (var i = 1; i < argv.Length; i++)
            psi.ArgumentList.Add(argv[i]);

        try
        {
            var process = _launch(psi);
            return Task.FromResult(new CommandResult
            {
                RequestId = command.RequestId,
                Success = process is not null
            });
        }
        catch (Exception ex)
        {
            return Task.FromResult(new CommandResult
            {
                RequestId = command.RequestId,
                Success = false,
                ErrorMessage = ex.Message
            });
        }
    }
}
