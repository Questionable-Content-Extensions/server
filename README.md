# QCExtensions.Server

## Usage

```bash
npm install
dotnet restore
dotnet run -p src/QCExtensions.Server/QCExtensions.Server.csproj
npm start
```

## Deploy to Heroku

### Manual

Using custom buildpack [dotnetcore-buildpack](https://github.com/jincod/dotnetcore-buildpack)

```bash
heroku buildpacks:set https://github.com/jincod/dotnetcore-buildpack
heroku buildpacks:add --index 1 heroku/nodejs
```
