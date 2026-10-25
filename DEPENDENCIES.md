# Dependencies

## C#

| Package | Version | Used in | Why | License |
|---|---|---|---|---|
| `xunit` | 2.9.2 | `Sentinel.Agent.Tests` | Test framework | Apache 2.0 |
| `xunit.runner.visualstudio` | 2.8.2 | `Sentinel.Agent.Tests` | Allows `dotnet test` to discover and run xUnit tests | MIT |
| `Microsoft.NET.Test.Sdk` | 17.12.0 | `Sentinel.Agent.Tests` | Required infrastructure for `dotnet test` | MIT |
| `coverlet.collector` | 6.0.2 | `Sentinel.Agent.Tests` | Code coverage collection during test runs | MIT |

## GitHub Actions

| Action | Version | Used in | Why | License |
|---|---|---|---|---|
| `actions/checkout` | v4 | `move-to-in-progress.yml` | Checks out the repo so workflows can access scripts in `.github/scripts/` | MIT |
