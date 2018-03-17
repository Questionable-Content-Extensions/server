#!/usr/bin/env bash
cd src/QCExtensions.Server/
dotnet restore
dotnet ef database update
