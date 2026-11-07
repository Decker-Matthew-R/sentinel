namespace Sentinel.Agent;

public class FakeSteamLibrary : ISteamLibrary
{
    public GameInfo? GetLastPlayedGame() => new GameInfo("570", "Dota 2");
}
