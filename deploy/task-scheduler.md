# Task Scheduler Setup — Sentinel Agent

Sentinel Agent must run elevated (Administrator) so LibreHardwareMonitorLib can load
the WinRing0 kernel driver for CPU temperature access. This Task Scheduler task starts
it automatically at logon without requiring a manual admin PowerShell session.

## Prerequisites

- Repo extracted to `C:\sentinel\` so the agent source is at `C:\sentinel\agent\`
- Agent built at least once (see build step below)
- USB tethering enabled on the tablet before or shortly after logon (the agent retries
  with backoff if the tablet is not ready yet)

## Step 1 — Build the agent

Open PowerShell **as Administrator** and run:

```powershell
dotnet build C:\sentinel\agent\src\Sentinel.Agent -c Release
```

The exe will be at:
```
C:\sentinel\agent\src\Sentinel.Agent\bin\Release\net9.0\sentinel-agent.exe
```

## Step 2 — Register the task

```powershell
$exe      = "C:\sentinel\agent\src\Sentinel.Agent\bin\Release\net9.0\sentinel-agent.exe"
$action   = New-ScheduledTaskAction -Execute $exe
$trigger  = New-ScheduledTaskTrigger -AtLogOn -User $env:USERNAME
$trigger.Delay = "PT15S"
$principal = New-ScheduledTaskPrincipal -UserId $env:USERNAME -LogonType Interactive -RunLevel Highest
$settings = New-ScheduledTaskSettingsSet `
    -ExecutionTimeLimit ([TimeSpan]::Zero) `
    -RestartCount 3 `
    -RestartInterval (New-TimeSpan -Minutes 1) `
    -MultipleInstances IgnoreNew
Register-ScheduledTask `
    -TaskName "SentinelAgent" `
    -TaskPath "\Sentinel" `
    -Action $action `
    -Trigger $trigger `
    -Principal $principal `
    -Settings $settings `
    -Description "Starts Sentinel Agent at logon with elevated privileges" `
    -Force
```

The `-Force` flag overwrites any existing task with the same name.

## Step 3 — Verify

- Open **Task Scheduler** → Task Scheduler Library → Sentinel → SentinelAgent
- Check that **Run As** shows your username and **Run Level** shows "Highest Privileges"
- Click **Run** in the right panel to test without rebooting
- After a few seconds, check the log:

```powershell
Get-Content "C:\sentinel\sentinel-agent.log"
```

You should see four lines ending with `Connecting to <IP>:9000`. The tablet should show live data.

- Reboot to confirm the task fires automatically at logon (allow ~20 seconds after login)

## Key settings explained

| Setting | Value | Why |
|---|---|---|
| Run Level | Highest Available | Required for WinRing0 kernel driver (CPU temps) |
| Logon Type | Interactive Token | Runs in your user session so it can launch Steam and desktop apps |
| Startup Delay | 15 seconds | Gives Windows time to bring up the RNDIS adapter before the agent looks for it |
| Execution Time Limit | None | The agent runs indefinitely |
| Multiple Instances | Ignore New | Prevents a second instance if Task Scheduler fires again |
| Restart on Failure | 3× at 1 min | Recovers from a crash before the tablet connects |

## When you need to redo this

- **New PC or Windows reinstall** — Task Scheduler entries don't survive. Repeat all three steps.
- **Switching tablets** — no changes needed. The agent discovers the tablet dynamically via RNDIS regardless of which tablet is connected.
- **Agent update** — rebuild (Step 1) and reboot. The task picks up the new exe automatically.

## Log file

The agent logs startup events to `C:\sentinel\sentinel-agent.log`. Check it if the tablet
isn't connecting — it will show exactly where the agent stopped (finding the hardware,
discovering the tablet IP, or connecting).

## Removing the task

```powershell
Unregister-ScheduledTask -TaskName "SentinelAgent" -TaskPath "\Sentinel" -Confirm:$false
```
