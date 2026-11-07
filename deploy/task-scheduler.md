# Task Scheduler Setup — Sentinel Agent

Sentinel Agent must run elevated (Administrator) so LibreHardwareMonitorLib can load
the WinRing0 kernel driver for CPU temperature access. This Task Scheduler task starts
it automatically at logon without requiring a manual admin PowerShell session.

## Prerequisites

- Agent deployed to `C:\sentinel\sentinel-agent.exe`
- USB tethering enabled on the tablet before or shortly after logon (the agent retries
  with backoff if the tablet is not ready yet)

## Option A — PowerShell (recommended, one command)

Open PowerShell **as Administrator** and run:

```powershell
$action   = New-ScheduledTaskAction -Execute "C:\sentinel\sentinel-agent.exe"
$trigger  = New-ScheduledTaskTrigger -AtLogOn -User $env:USERNAME
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
    -Description "Starts Sentinel Agent at logon with elevated privileges"
```

This registers the task for the current user automatically — no username editing needed.

## Option B — Import the XML file

1. Open `deploy/SentinelAgent.xml` in a text editor and replace both occurrences of
   `YOUR_PC_USERNAME` with your Windows username. To find it, run `echo %USERNAME%`
   in a Command Prompt.

2. Open **Task Scheduler** (search for it in the Start menu).

3. In the left panel, right-click **Task Scheduler Library** and choose
   **Import Task…**

4. Select the edited `SentinelAgent.xml` and click Open.

5. Click **OK** to save.

## Verifying the task

- In Task Scheduler, find **Task Scheduler Library → Sentinel → SentinelAgent**.
- Check that **Run As** shows your username and **Run Level** shows "Highest Privileges".
- Click **Run** in the right panel to test it immediately without rebooting.
- Confirm the agent connects to the tablet and sensor data appears.

## Key settings explained

| Setting | Value | Why |
|---|---|---|
| Run Level | Highest Available | Required for WinRing0 kernel driver (CPU temps) |
| Logon Type | Interactive Token | Runs in your user session so it can launch Steam and desktop apps |
| Execution Time Limit | None | The agent runs indefinitely |
| Multiple Instances | Ignore New | Prevents a second instance if Task Scheduler fires again |
| Restart on Failure | 3× at 1 min | Recovers from a crash before the tablet connects |

## Updating the agent

When a new `sentinel-agent.exe` is placed at `C:\sentinel\`, the task picks it up
automatically on the next logon — no changes to the task needed.

## Removing the task

```powershell
Unregister-ScheduledTask -TaskName "SentinelAgent" -TaskPath "\Sentinel" -Confirm:$false
```
