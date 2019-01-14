.\set-dev-env.ps1
dotnet restore

Push-Location test/QCExtensions.Domain.Tests
dotnet test
Pop-Location

Push-Location test/QCExtensions.Application.Tests
dotnet test
Pop-Location

Push-Location test/QCExtensions.Server.Test
dotnet test
Pop-Location
