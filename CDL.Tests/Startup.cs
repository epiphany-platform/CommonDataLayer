using CDL.Tests.Configuration;
using CDL.Tests.MessageBroker.Kafka;
using MassTransit;
using MassTransit.KafkaIntegration;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using CDL.Tests.Services;
using static SchemaRegistry.SchemaRegistry;
using System;
using static EdgeRegistry.EdgeRegistry;
using EdgeRegistry;
using static MaterializerOndemand.OnDemandMaterializer;
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
                    rider.AddProducer<InsertEdgeObject>(configuration.CDL_KAFKA_EDGE_INPUT_TOPIC);
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
            services.AddGrpcClient<EdgeRegistryClient>(o =>
            {
                o.Address = new Uri(configuration.CDL_EDGE_REGISTRY_ADDRESS);
            });
            services.AddGrpcClient<OnDemandMaterializerClient>(o =>
            {
                o.Address = new Uri(configuration.CDL_MATERIALIZER_ONDEMAND_ADDRESS);
            });
            services.AddScoped<EdgeRegistryService>();
            services.AddScoped<SchemaRegistryService>();
            services.AddScoped<QueryRouterService>();
            services.AddScoped<OnDemandMaterializerService>();
            services.AddScoped<Fixture>();
        } 

        // public void ConfigureHost(IHostBuilder hostBuilder)
        // {

        // }

        // public void Configure(IServiceProvider applicationServices)
        // {

        // }
    }
}