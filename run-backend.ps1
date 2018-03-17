.\set-dev-env.ps1
dotnet restore
Push-Location src/QCExtensions.Server
try {
	dotnet watch run
}
finally {
	Pop-Location
}
