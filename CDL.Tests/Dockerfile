FROM mcr.microsoft.com/dotnet/sdk:5.0
WORKDIR /App
COPY /publish .
ENTRYPOINT ["dotnet", "vstest", "CDL.Tests.dll"]