FROM mcr.microsoft.com/dotnet/sdk:5.0
WORKDIR /App
COPY . .
RUN dotnet test
ENTRYPOINT ["dotnet", "test"]