#!/usr/bin/env bash

dotnet restore
dotnet build -o './publish'

docker build -t cdl-tests:latest .