using Sentinel.V1;

namespace Sentinel.Agent.Tests;

public class FakeActionRunner : IActionRunner
{
    public Task<CommandResult> ExecuteAsync(Command command) =>
        Task.FromResult(new CommandResult { RequestId = command.RequestId, Success = true });
}
