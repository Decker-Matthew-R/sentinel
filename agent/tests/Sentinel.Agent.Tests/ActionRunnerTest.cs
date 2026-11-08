using System.Diagnostics;
using Sentinel.V1;

namespace Sentinel.Agent.Tests;

public class ActionRunnerTest
{
    private static ActionRunner MakeRunner(
        Dictionary<string, string[]>? actions = null,
        Func<ProcessStartInfo, Process?>? launcher = null)
    {
        actions ??= new Dictionary<string, string[]> { ["open-notepad"] = ["notepad.exe"] };
        launcher ??= _ => new Process();
        return new ActionRunner(actions, launcher);
    }

    [Fact]
    public async Task ExecuteAsync_known_action_returns_success()
    {
        var runner = MakeRunner();
        var result = await runner.ExecuteAsync(new Command { RequestId = "1", ActionId = "open-notepad" });
        Assert.True(result.Success);
        Assert.Equal("1", result.RequestId);
    }

    [Fact]
    public async Task ExecuteAsync_unknown_action_returns_error_without_launching()
    {
        var launched = false;
        var runner = MakeRunner(launcher: _ => { launched = true; return null; });
        var result = await runner.ExecuteAsync(new Command { RequestId = "2", ActionId = "unknown-action" });
        Assert.False(result.Success);
        Assert.False(launched);
        Assert.Contains("unknown-action", result.ErrorMessage);
    }

    [Fact]
    public async Task ExecuteAsync_launch_throws_returns_error_with_message()
    {
        var runner = MakeRunner(launcher: _ => throw new InvalidOperationException("access denied"));
        var result = await runner.ExecuteAsync(new Command { RequestId = "3", ActionId = "open-notepad" });
        Assert.False(result.Success);
        Assert.Equal("access denied", result.ErrorMessage);
    }

    [Fact]
    public async Task ExecuteAsync_passes_arguments_to_process_start_info()
    {
        ProcessStartInfo? captured = null;
        var actions = new Dictionary<string, string[]> { ["launch-game"] = ["steam.exe", "-applaunch", "123"] };
        var runner = new ActionRunner(actions, psi => { captured = psi; return null; });

        await runner.ExecuteAsync(new Command { RequestId = "4", ActionId = "launch-game" });

        Assert.NotNull(captured);
        Assert.Equal("steam.exe", captured.FileName);
        Assert.Equal(["-applaunch", "123"], captured.ArgumentList.ToArray());
    }

    [Fact]
    public async Task ExecuteAsync_steam_uri_uses_shell_execute()
    {
        ProcessStartInfo? captured = null;
        var actions = new Dictionary<string, string[]> { ["launch_last_game"] = ["steam://rungameid/570"] };
        var runner = new ActionRunner(actions, psi => { captured = psi; return null; });

        await runner.ExecuteAsync(new Command { RequestId = "6", ActionId = "launch_last_game" });

        Assert.NotNull(captured);
        Assert.True(captured!.UseShellExecute);
        Assert.Equal("steam://rungameid/570", captured.FileName);
        Assert.Empty(captured.ArgumentList);
    }

    [Fact]
    public async Task ExecuteAsync_similar_id_with_trailing_space_does_not_match()
    {
        var launched = false;
        var runner = MakeRunner(launcher: _ => { launched = true; return null; });
        var result = await runner.ExecuteAsync(new Command { RequestId = "5", ActionId = "open-notepad " });
        Assert.False(result.Success);
        Assert.False(launched);
    }
}
