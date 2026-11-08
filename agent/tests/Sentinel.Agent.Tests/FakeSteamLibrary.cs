namespace Sentinel.Agent;

public class FakeSteamLibrary : ISteamLibrary
{
    public GameInfo? GetLastPlayedGame() => new GameInfo("570", "Dota 2", []);

    public IEnumerable<GameInfo> GetInstalledGames() =>
    [
        new GameInfo("570", "Dota 2", []),
        new GameInfo("730", "Counter-Strike 2", []),
        new GameInfo("440", "Team Fortress 2", []),
    ];
}
