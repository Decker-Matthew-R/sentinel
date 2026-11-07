using System.Runtime.Versioning;
using Microsoft.Win32;

namespace Sentinel.Agent;

public class SteamLibrary(string steamPath) : ISteamLibrary
{
    [SupportedOSPlatform("windows")]
    public static string? FindSteamPath() =>
        Registry.GetValue(@"HKEY_CURRENT_USER\Software\Valve\Steam", "SteamPath", null) as string;

    public GameInfo? GetLastPlayedGame()
    {
        var appId = FindLastPlayedAppId();
        if (appId is null) return null;
        var name = FindAppName(appId);
        if (name is null) return null;
        var coverArt = LoadCoverArt(appId);
        return new GameInfo(appId, name, coverArt);
    }

    private byte[] LoadCoverArt(string appId)
    {
        var dir = Path.Combine(steamPath, "appcache", "librarycache", appId);
        if (!Directory.Exists(dir)) return [];
        try
        {
            var files = Directory.GetFiles(dir, "library_capsule.jpg", SearchOption.AllDirectories);
            return files.Length > 0 ? File.ReadAllBytes(files[0]) : [];
        }
        catch { return []; }
    }

    public IEnumerable<GameInfo> GetInstalledGames()
    {
        var steamappsPath = Path.Combine(steamPath, "steamapps");
        if (!Directory.Exists(steamappsPath)) return [];
        var result = new List<GameInfo>();
        foreach (var manifest in Directory.GetFiles(steamappsPath, "appmanifest_*.acf"))
        {
            try
            {
                var text = File.ReadAllText(manifest);
                var name = ParseNameFromManifest(text);
                if (name is null) continue;
                var appId = Path.GetFileNameWithoutExtension(manifest)["appmanifest_".Length..];
                var coverArt = LoadCoverArt(appId);
                result.Add(new GameInfo(appId, name, coverArt));
            }
            catch { }
        }
        return result;
    }

    private string? FindLastPlayedAppId()
    {
        var userdataPath = Path.Combine(steamPath, "userdata");
        if (!Directory.Exists(userdataPath)) return null;

        string? bestAppId = null;
        long bestTimestamp = 0;

        foreach (var userDir in Directory.GetDirectories(userdataPath))
        {
            var configPath = Path.Combine(userDir, "config", "localconfig.vdf");
            if (!File.Exists(configPath)) continue;
            try
            {
                var (appId, timestamp) = ParseLastPlayedFromLocalConfig(File.ReadAllText(configPath));
                if (appId is not null && timestamp > bestTimestamp)
                {
                    bestAppId = appId;
                    bestTimestamp = timestamp;
                }
            }
            catch { }
        }

        return bestAppId;
    }

    private string? FindAppName(string appId)
    {
        var manifestPath = Path.Combine(steamPath, "steamapps", $"appmanifest_{appId}.acf");
        if (!File.Exists(manifestPath)) return null;
        try
        {
            return ParseNameFromManifest(File.ReadAllText(manifestPath));
        }
        catch { return null; }
    }

    public static (string? AppId, long Timestamp) ParseLastPlayedFromLocalConfig(string content)
    {
        try
        {
            if (VdfParser.Parse(content) is not VdfMap root) return (null, 0);
            if (root.Children["Software"] is not VdfMap software) return (null, 0);
            if (software.Children["Valve"] is not VdfMap valve) return (null, 0);
            if (valve.Children["Steam"] is not VdfMap steam) return (null, 0);
            if (!steam.Children.TryGetValue("apps", out var appsNode) || appsNode is not VdfMap apps)
                return (null, 0);

            string? bestAppId = null;
            long bestTimestamp = 0;

            foreach (var (appId, appNode) in apps.Children)
            {
                if (appNode is not VdfMap appMap) continue;
                if (!appMap.Children.TryGetValue("LastPlayed", out var lastPlayedNode)) continue;
                if (lastPlayedNode is not VdfString lastPlayedStr) continue;
                if (!long.TryParse(lastPlayedStr.Value, out var timestamp)) continue;
                if (timestamp > bestTimestamp)
                {
                    bestTimestamp = timestamp;
                    bestAppId = appId;
                }
            }

            return (bestAppId, bestTimestamp);
        }
        catch { return (null, 0); }
    }

    public static string? ParseNameFromManifest(string content)
    {
        try
        {
            if (VdfParser.Parse(content) is not VdfMap root) return null;
            return root.Children.TryGetValue("name", out var nameNode) && nameNode is VdfString nameStr
                ? nameStr.Value
                : null;
        }
        catch { return null; }
    }
}
