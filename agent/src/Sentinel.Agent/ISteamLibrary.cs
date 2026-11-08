namespace Sentinel.Agent;

public interface ISteamLibrary
{
    GameInfo? GetLastPlayedGame();
    IEnumerable<GameInfo> GetInstalledGames();
}
