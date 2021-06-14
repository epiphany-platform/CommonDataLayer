using CDL.Tests.Configuration;
using MassTransit;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;

namespace CDL.Tests
{
    public class Startup
    {        
        public Startup()
        {
            var configuration = new ConfigurationBuilder()
                .AddEnvironmentVariables()
                .Build();
            this.Configuration = configuration;
        }

        public IConfiguration Configuration { get; }

        public void ConfigureServices(IServiceCollection services)
        {
            services.Configure<ConfigurationOptions>(options => this.Configuration.Bind(options));
            var configuration = this.Configuration.Get<ConfigurationOptions>();            

            services.AddMassTransit(x => 
            {
                x.UsingRabbitMq((context, cfg) => cfg.ConfigureEndpoints(context));
                x.AddRider(rider => {
                    rider.UsingKafka((context, k) => {
                        k.Host(configuration.);
                    });
                });

            });
            services.AddSingleton<DependencyClass>();
            services.AddSingleton<DependencyClassJ>();
        } 

        // public void ConfigureHost(IHostBuilder hostBuilder)
        // {

        // }

        // public void Configure(IServiceProvider applicationServices)
        // {

        // }
    }
}