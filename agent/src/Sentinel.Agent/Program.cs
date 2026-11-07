using System.Net.NetworkInformation;
using LibreHardwareMonitor.Hardware;
using Sentinel.Agent;

var log = new StreamWriter(@"C:\sentinel\sentinel-agent.log", append: true) { AutoFlush = true };
Console.SetOut(log);
Console.SetError(log);
Console.WriteLine($"[{DateTime.Now:yyyy-MM-dd HH:mm:ss}] Sentinel Agent 0.2.0 starting");

var computer = new Computer
{
    IsCpuEnabled = true,
    IsGpuEnabled = true,
    IsMemoryEnabled = true,
    IsMotherboardEnabled = true,
};
computer.Open();
Console.WriteLine($"[{DateTime.Now:yyyy-MM-dd HH:mm:ss}] Hardware monitor opened");

var cts = new CancellationTokenSource();
Console.CancelKeyPress += (_, e) =>
{
    e.Cancel = true;
    cts.Cancel();
};

var sensorSource = new LibreHardwareMonitorSource(computer);
var specsSource = new WindowsSpecsSource(computer);
var poller = new SensorPoller(sensorSource);

var actions = new Dictionary<string, string[]>
{
    ["launch_steam"] = [@"C:\Program Files (x86)\Steam\steam.exe"],
};

ISteamLibrary? steamLibrary = null;
if (OperatingSystem.IsWindows())
{
    var steamPath = SteamLibrary.FindSteamPath();
    if (steamPath is not null)
    {
        steamLibrary = new SteamLibrary(steamPath);
        var game = steamLibrary.GetLastPlayedGame();
        if (game is not null)
        {
            actions["launch_last_game"] = [$"steam://rungameid/{game.AppId}"];
            Console.WriteLine($"[{DateTime.Now:yyyy-MM-dd HH:mm:ss}] Last played game: {game.Name} ({game.AppId})");
        }
    }
}

var actionRunner = new ActionRunner(actions);

string host;
if (args.Length > 0)
{
    host = args[0];
    Console.WriteLine($"[{DateTime.Now:yyyy-MM-dd HH:mm:ss}] Using host from argument: {host}");
}
else
{
    host = FindTabletHost() ?? throw new InvalidOperationException(
        "Could not find tablet: no RNDIS adapter with a gateway address. " +
        "Enable USB tethering on the tablet, or pass the IP as an argument.");
    Console.WriteLine($"[{DateTime.Now:yyyy-MM-dd HH:mm:ss}] Tablet discovered at {host}");
}

var client = new LinkClient(host, 9000, poller, specsSource, actionRunner, steamLibrary);
Console.WriteLine($"[{DateTime.Now:yyyy-MM-dd HH:mm:ss}] Connecting to {host}:9000");

await client.RunWithReconnectAsync(cts.Token);

Console.WriteLine($"[{DateTime.Now:yyyy-MM-dd HH:mm:ss}] Sentinel Agent stopped");
computer.Close();

static string? FindTabletHost() =>
    NetworkInterface.GetAllNetworkInterfaces()
        .Where(ni => ni.OperationalStatus == OperationalStatus.Up
                  && ni.Description.Contains("NDIS", StringComparison.OrdinalIgnoreCase))
        .SelectMany(ni => ni.GetIPProperties().GatewayAddresses)
        .Select(ga => ga.Address.ToString())
        .FirstOrDefault(ip => ip != "0.0.0.0");
