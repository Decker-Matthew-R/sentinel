using System.Runtime.Versioning;

namespace Sentinel.Agent.Tests;

public class SteamLibraryTest
{
    [Fact]
    public void ParseNameFromManifest_returns_name_from_appmanifest()
    {
        var vdf = """
            "AppState"
            {
                "appid"      "570"
                "name"       "Dota 2"
                "StateFlags" "4"
            }
            """;
        Assert.Equal("Dota 2", SteamLibrary.ParseNameFromManifest(vdf));
    }

    [Fact]
    public void ParseNameFromManifest_returns_null_when_name_key_missing()
    {
        var vdf = """
            "AppState"
            {
                "appid" "570"
            }
            """;
        Assert.Null(SteamLibrary.ParseNameFromManifest(vdf));
    }

    [Fact]
    public void ParseLastPlayedFromLocalConfig_returns_most_recently_played_app()
    {
        var vdf = """
            "UserLocalConfigStore"
            {
                "Software"
                {
                    "Valve"
                    {
                        "Steam"
                        {
                            "apps"
                            {
                                "570"
                                {
                                    "LastPlayed" "1700000000"
                                }
                                "440"
                                {
                                    "LastPlayed" "1700000100"
                                }
                            }
                        }
                    }
                }
            }
            """;
        var (appId, _) = SteamLibrary.ParseLastPlayedFromLocalConfig(vdf);
        Assert.Equal("440", appId);
    }

    [Fact]
    public void ParseLastPlayedFromLocalConfig_returns_null_when_apps_section_missing()
    {
        var vdf = """
            "UserLocalConfigStore"
            {
                "Software"
                {
                    "Valve"
                    {
                        "Steam"
                        {
                        }
                    }
                }
            }
            """;
        var (appId, _) = SteamLibrary.ParseLastPlayedFromLocalConfig(vdf);
        Assert.Null(appId);
    }

    [Fact]
    public void ParseLastPlayedFromLocalConfig_returns_null_when_no_apps_have_LastPlayed()
    {
        var vdf = """
            "UserLocalConfigStore"
            {
                "Software"
                {
                    "Valve"
                    {
                        "Steam"
                        {
                            "apps"
                            {
                                "570"
                                {
                                    "SomeOtherKey" "value"
                                }
                            }
                        }
                    }
                }
            }
            """;
        var (appId, _) = SteamLibrary.ParseLastPlayedFromLocalConfig(vdf);
        Assert.Null(appId);
    }

    [Fact]
    public void LoadCoverArt_returns_bytes_when_flat_library_600x900_exists()
    {
        var steamPath = Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString());
        try
        {
            var cacheDir = Path.Combine(steamPath, "appcache", "librarycache", "934700");
            Directory.CreateDirectory(cacheDir);
            var imageBytes = new byte[] { 0xFF, 0xD8, 0xFF };
            File.WriteAllBytes(Path.Combine(cacheDir, "library_600x900.jpg"), imageBytes);

            var steamappsDir = Path.Combine(steamPath, "steamapps");
            Directory.CreateDirectory(steamappsDir);
            File.WriteAllText(Path.Combine(steamappsDir, "appmanifest_934700.acf"), """
                "AppState"
                {
                    "appid" "934700"
                    "name" "Dead Island 2"
                    "type" "game"
                }
                """);

            var games = new SteamLibrary(steamPath).GetInstalledGames().ToList();

            Assert.Single(games);
            Assert.Equal(imageBytes, games[0].CoverArt);
        }
        finally
        {
            Directory.Delete(steamPath, recursive: true);
        }
    }

    [Fact]
    public void LoadCoverArt_returns_bytes_when_library_600x900_in_hash_subdir_exists()
    {
        var steamPath = Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString());
        try
        {
            var hashDir = Path.Combine(steamPath, "appcache", "librarycache", "552520", "abc123");
            Directory.CreateDirectory(hashDir);
            var imageBytes = new byte[] { 0xFF, 0xD8, 0xFF };
            File.WriteAllBytes(Path.Combine(hashDir, "library_600x900.jpg"), imageBytes);

            var steamappsDir = Path.Combine(steamPath, "steamapps");
            Directory.CreateDirectory(steamappsDir);
            File.WriteAllText(Path.Combine(steamappsDir, "appmanifest_552520.acf"), """
                "AppState"
                {
                    "appid" "552520"
                    "name" "Far Cry 5"
                    "type" "game"
                }
                """);

            var games = new SteamLibrary(steamPath).GetInstalledGames().ToList();

            Assert.Single(games);
            Assert.Equal(imageBytes, games[0].CoverArt);
        }
        finally
        {
            Directory.Delete(steamPath, recursive: true);
        }
    }

    [Fact]
    public void ParseTypeFromManifest_returns_type_from_appmanifest()
    {
        var vdf = """
            "AppState"
            {
                "appid" "228980"
                "name" "Steamworks Common Redistributables"
                "type" "Tool"
            }
            """;
        Assert.Equal("Tool", SteamLibrary.ParseTypeFromManifest(vdf));
    }

    [Fact]
    public void ParseTypeFromManifest_returns_null_when_type_key_missing()
    {
        var vdf = """
            "AppState"
            {
                "appid" "570"
                "name" "Dota 2"
            }
            """;
        Assert.Null(SteamLibrary.ParseTypeFromManifest(vdf));
    }

    [Fact]
    public void GetInstalledGames_excludes_tool_type_entries()
    {
        var steamPath = Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString());
        try
        {
            var steamappsDir = Path.Combine(steamPath, "steamapps");
            Directory.CreateDirectory(steamappsDir);
            File.WriteAllText(Path.Combine(steamappsDir, "appmanifest_570.acf"), """
                "AppState"
                {
                    "appid" "570"
                    "name" "Dota 2"
                    "type" "Game"
                }
                """);
            File.WriteAllText(Path.Combine(steamappsDir, "appmanifest_228980.acf"), """
                "AppState"
                {
                    "appid" "228980"
                    "name" "Steamworks Common Redistributables"
                    "type" "Tool"
                }
                """);

            var games = new SteamLibrary(steamPath).GetInstalledGames().ToList();

            Assert.Single(games);
            Assert.Equal("Dota 2", games[0].Name);
        }
        finally
        {
            Directory.Delete(steamPath, recursive: true);
        }
    }

    [Fact]
    public void LoadCoverArt_falls_back_to_library_capsule_when_library_600x900_absent()
    {
        var steamPath = Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString());
        try
        {
            var hashDir = Path.Combine(steamPath, "appcache", "librarycache", "552520", "abc123");
            Directory.CreateDirectory(hashDir);
            var imageBytes = new byte[] { 0xFF, 0xD8, 0xFF };
            File.WriteAllBytes(Path.Combine(hashDir, "library_capsule.jpg"), imageBytes);

            var steamappsDir = Path.Combine(steamPath, "steamapps");
            Directory.CreateDirectory(steamappsDir);
            File.WriteAllText(Path.Combine(steamappsDir, "appmanifest_552520.acf"), """
                "AppState"
                {
                    "appid" "552520"
                    "name" "Far Cry 5"
                    "type" "game"
                }
                """);

            var games = new SteamLibrary(steamPath).GetInstalledGames().ToList();

            Assert.Single(games);
            Assert.Equal(imageBytes, games[0].CoverArt);
        }
        finally
        {
            Directory.Delete(steamPath, recursive: true);
        }
    }

    [Fact]
    public void GetInstalledGames_excludes_entries_without_type_field()
    {
        var steamPath = Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString());
        try
        {
            var steamappsDir = Path.Combine(steamPath, "steamapps");
            Directory.CreateDirectory(steamappsDir);
            File.WriteAllText(Path.Combine(steamappsDir, "appmanifest_570.acf"), """
                "AppState"
                {
                    "appid" "570"
                    "name" "Dota 2"
                    "type" "game"
                }
                """);
            File.WriteAllText(Path.Combine(steamappsDir, "appmanifest_228980.acf"), """
                "AppState"
                {
                    "appid" "228980"
                    "name" "Steamworks Common Redistributables"
                }
                """);

            var games = new SteamLibrary(steamPath).GetInstalledGames().ToList();

            Assert.Single(games);
            Assert.Equal("Dota 2", games[0].Name);
        }
        finally
        {
            Directory.Delete(steamPath, recursive: true);
        }
    }

    [Fact(Skip = "Requires Windows with Steam installed — run manually on the target PC")]
    [SupportedOSPlatform("windows")]
    public void SteamLibrary_finds_real_last_played_game()
    {
        var steamPath = SteamLibrary.FindSteamPath();
        Assert.NotNull(steamPath);
        var library = new SteamLibrary(steamPath);
        var game = library.GetLastPlayedGame();
        Assert.NotNull(game);
        Assert.NotEmpty(game.AppId);
        Assert.NotEmpty(game.Name);
    }
}
