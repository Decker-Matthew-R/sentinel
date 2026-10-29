namespace Sentinel.Agent.Tests;

public class VdfParserTest
{
    [Fact]
    public void Parse_empty_map_returns_VdfMap_with_no_children()
    {
        var result = VdfParser.Parse("""
            "root"
            {
            }
            """);
        var map = Assert.IsType<VdfMap>(result);
        Assert.Empty(map.Children);
    }

    [Fact]
    public void Parse_single_string_value_returns_correct_leaf()
    {
        var result = VdfParser.Parse("""
            "AppState"
            {
                "name" "Dota 2"
            }
            """);
        var map = Assert.IsType<VdfMap>(result);
        var name = Assert.IsType<VdfString>(map.Children["name"]);
        Assert.Equal("Dota 2", name.Value);
    }

    [Fact]
    public void Parse_multiple_keys_returns_all_entries()
    {
        var result = VdfParser.Parse("""
            "AppState"
            {
                "appid"      "570"
                "name"       "Dota 2"
                "StateFlags" "4"
            }
            """);
        var map = Assert.IsType<VdfMap>(result);
        Assert.Equal(3, map.Children.Count);
        Assert.Equal("570", ((VdfString)map.Children["appid"]).Value);
        Assert.Equal("Dota 2", ((VdfString)map.Children["name"]).Value);
        Assert.Equal("4", ((VdfString)map.Children["StateFlags"]).Value);
    }

    [Fact]
    public void Parse_nested_map_is_parsed_correctly()
    {
        var result = VdfParser.Parse("""
            "AppState"
            {
                "details"
                {
                    "appid" "570"
                }
            }
            """);
        var root = Assert.IsType<VdfMap>(result);
        var details = Assert.IsType<VdfMap>(root.Children["details"]);
        var appid = Assert.IsType<VdfString>(details.Children["appid"]);
        Assert.Equal("570", appid.Value);
    }

    [Fact]
    public void Parse_comments_are_skipped()
    {
        var result = VdfParser.Parse("""
            "root"
            {
                // this is a comment
                "key" "value"
            }
            """);
        var map = Assert.IsType<VdfMap>(result);
        Assert.Single(map.Children);
        Assert.Equal("value", ((VdfString)map.Children["key"]).Value);
    }

    [Fact]
    public void Parse_children_lookup_is_case_insensitive()
    {
        var result = VdfParser.Parse("""
            "AppState"
            {
                "AppID" "570"
            }
            """);
        var map = Assert.IsType<VdfMap>(result);
        Assert.True(map.Children.ContainsKey("appid"));
        Assert.True(map.Children.ContainsKey("APPID"));
        Assert.True(map.Children.ContainsKey("AppID"));
    }

    [Fact]
    public void Parse_realistic_appmanifest_snippet_extracts_name_and_appid()
    {
        var result = VdfParser.Parse("""
            "AppState"
            {
                "appid"      "570"
                "name"       "Dota 2"
                "StateFlags" "4"
                "installdir" "Dota 2"
                "SizeOnDisk" "29384293840"
            }
            """);
        var root = Assert.IsType<VdfMap>(result);
        Assert.Equal("570", ((VdfString)root.Children["appid"]).Value);
        Assert.Equal("Dota 2", ((VdfString)root.Children["name"]).Value);
    }

    [Fact]
    public void Parse_deeply_nested_structure_is_traversable()
    {
        var result = VdfParser.Parse("""
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
                            }
                        }
                    }
                }
            }
            """);
        var root = Assert.IsType<VdfMap>(result);
        var software = Assert.IsType<VdfMap>(root.Children["Software"]);
        var valve = Assert.IsType<VdfMap>(software.Children["Valve"]);
        var steam = Assert.IsType<VdfMap>(valve.Children["Steam"]);
        var apps = Assert.IsType<VdfMap>(steam.Children["apps"]);
        var game = Assert.IsType<VdfMap>(apps.Children["570"]);
        var lastPlayed = Assert.IsType<VdfString>(game.Children["LastPlayed"]);
        Assert.Equal("1700000000", lastPlayed.Value);
    }

    [Fact]
    public void Parse_throws_on_unclosed_brace()
    {
        Assert.Throws<FormatException>(() => VdfParser.Parse("""
            "root"
            {
                "key" "value"
            """));
    }

    [Fact]
    public void Parse_throws_on_empty_input()
    {
        Assert.Throws<FormatException>(() => VdfParser.Parse(""));
    }
}
