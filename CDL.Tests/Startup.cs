using CDL.Tests.Configuration;
using CDL.Tests.MessageBroker.Kafka;
using MassTransit;
using MassTransit.KafkaIntegration;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using CDL.Tests.GrpcServices;
using static SchemaRegistry.SchemaRegistry;
using System;
using AutoFixture;

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
                    rider.AddProducer<InsertObject>(configuration.CDL_KAFKA_DATA_INPUT_TOPIC);
                    rider.UsingKafka((context, k) => {
                        k.Host(configuration.CDL_KAFKA_BROKER);
                    });
                });
            });
            services.AddMassTransitHostedService();

            services.AddGrpcClient<SchemaRegistryClient>(o =>
            {
                o.Address = new Uri(configuration.CDL_SCHEMA_REGISTRY_ADDRESS);
            });

            services.AddTransient<SchemaRegistryService>();
            services.AddTransient<Fixture>();

        } 

        // public void ConfigureHost(IHostBuilder hostBuilder)
        // {

        // }

        // public void Configure(IServiceProvider applicationServices)
        // {

        // }
    }
}