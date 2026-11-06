using Sentinel.V1;

namespace Sentinel.Agent;

public interface IActionRunner
{
    Task<CommandResult> ExecuteAsync(Command command);
}
