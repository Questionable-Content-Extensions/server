.\set-dev-env.ps1
dotnet restore

Push-Location test/QCExtensions.Server.Test
dotnet xunit
Pop-Location
