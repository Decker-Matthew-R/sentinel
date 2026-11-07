using System.Net.NetworkInformation;
using LibreHardwareMonitor.Hardware;
using Sentinel.Agent;

var computer = new Computer
{
    IsCpuEnabled = true,
    IsGpuEnabled = true,
    IsMemoryEnabled = true,
    IsMotherboardEnabled = true,
};
computer.Open();

var cts = new CancellationTokenSource();
Console.CancelKeyPress += (_, e) =>
{
    e.Cancel = true;
    cts.Cancel();
};

var sensorSource = new LibreHardwareMonitorSource(computer);
var specsSource = new WindowsSpecsSource(computer);
var actionRunner = new ActionRunner(new Dictionary<string, string[]>
{
    ["launch_steam"] = [@"C:\Program Files (x86)\Steam\steam.exe"],
});
var poller = new SensorPoller(sensorSource);
var host = args.Length > 0 ? args[0] : FindTabletHost() ?? throw new InvalidOperationException(
    "Could not find tablet: no RNDIS adapter with a gateway address. " +
    "Enable USB tethering on the tablet, or pass the IP as an argument.");
var client = new LinkClient(host, 9000, poller, specsSource, actionRunner);

await client.RunWithReconnectAsync(cts.Token);

computer.Close();

static string? FindTabletHost() =>
    NetworkInterface.GetAllNetworkInterfaces()
        .Where(ni => ni.OperationalStatus == OperationalStatus.Up
                  && ni.Description.Contains("NDIS", StringComparison.OrdinalIgnoreCase))
        .SelectMany(ni => ni.GetIPProperties().GatewayAddresses)
        .Select(ga => ga.Address.ToString())
        .FirstOrDefault(ip => ip != "0.0.0.0");
