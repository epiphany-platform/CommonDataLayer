#!/usr/bin/env bash

for f in ls -l ../crates/rpc/proto/
do 
   cp "$f" ./"${f}"
done


dotnet restore
dotnet build -o './publish'

docker build -t cdl-tests:latest .