using System;
using System.Threading;
using System.Threading.Tasks;
using CDL.Tests.Configuration;
using CDL.Tests.MessageBroker.Kafka;
using MassTransit;
using MassTransit.KafkaIntegration;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.Options;
using Xunit;

namespace CDL.Tests
{
    public class UnitTest1
    {
        private readonly ConfigurationOptions _options;
        private ITopicProducer<InsertObject> _producer;

        public UnitTest1(IOptions<ConfigurationOptions> options, ITopicProducer<InsertObject> producer)
        {
            _options = options.Value;
            _producer = producer;            
        }

        [Fact]
        public async Task Test1()
        {
            var cancellationToken = new CancellationTokenSource(TimeSpan.FromSeconds(10)).Token;
            try
            {

                await _producer.Produce(new InsertObject(), cancellationToken);
            }
            catch (System.Exception e)
            {
                var a = e.InnerException;
                throw;
            }
            
            Assert.Contains("VALUE", _options.CDL_COMMAND_SERVICE_ADDRESS);
            Assert.Contains("VALUE", _options.CDL_DATA_ROUTER_ADDRESS);
            Assert.Contains("VALUE", _options.CDL_EDGE_REGISTRY_ADDRESS);
            Assert.Contains("VALUE", _options.CDL_MATERIALIZER_GENERAL_ADDRESS);
            Assert.Contains("VALUE", _options.CDL_MATERIALIZER_ONDEMAND_ADDRESS);
            Assert.Contains("VALUE", _options.CDL_QUERY_ROUTER_ADDRESS);
            Assert.Contains("VALUE", _options.CDL_QUERY_SERVICE_ADDRESS);
            Assert.Contains("VALUE", _options.CDL_SCHEMA_REGISTRY_ADDRESS);
        }
    }
}
