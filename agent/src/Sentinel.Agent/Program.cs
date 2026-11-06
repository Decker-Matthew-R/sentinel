using LibreHardwareMonitor.Hardware;
using Sentinel.Agent;

var computer = new Computer
{
    IsCpuEnabled = true,
    IsGpuEnabled = true,
    IsMemoryEnabled = true,
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
var client = new LinkClient("10.186.188.235", 9000, poller, specsSource, actionRunner);

await client.RunWithReconnectAsync(cts.Token);

computer.Close();
