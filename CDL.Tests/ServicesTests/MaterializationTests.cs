using System;
using AutoFixture;
using CDL.Tests.MessageBroker.Kafka;
using CDL.Tests.Services;
using CDL.Tests.TestDataObjects;
using MassTransit.KafkaIntegration;
using SchemaRegistry;
using Xunit;

namespace CDL.Tests.ServicesTests
{
    public class MaterializationTests
    {
        private SchemaRegistryService _schemaRegistryService;
        private QueryRouterService _queryService;
        private ITopicProducer<InsertObject> _kafkaProducer;
        private Fixture _fixture;

        public MaterializationTests(SchemaRegistryService schemaRegistryService, QueryRouterService queryService, ITopicProducer<InsertObject> kafkaProducer, Fixture fixture)
        {
            _schemaRegistryService = schemaRegistryService;
            _queryService = queryService;
            _kafkaProducer = kafkaProducer;
            _fixture = fixture;
        }

        [Fact]
        public void TestName()
        {
            var name = _fixture.Create<string>();
            var viewName = _fixture.Create<string>();            
            var schema = _schemaRegistryService.AddSchema(name, _fixture.Create<GeneralObject>().ToJSONString(), new SchemaType() { SchemaType_ = SchemaType.Types.Type.DocumentStorage }).Result;
            var view = _schemaRegistryService.AddViewToSchema(schema.Id_, viewName, "{\"Name\": \"Name\" }").Result;
            var viewDetails = _schemaRegistryService.GetView(view.Id_).Result;
            var schemaWithView = _schemaRegistryService.GetFullSchema(schema.Id_).Result;
            Assert.Equal(1, schemaWithView.Views.Count);
            
            var viewObject = schemaWithView.Views[0];
            Assert.Equal(viewName, viewObject.Name);
            Assert.IsType<string>(viewObject.Id);
            Assert.NotEqual("{\"Name\": \"Name\" }", viewObject.MaterializerOptions);
            Assert.NotNull(viewDetails);
        }

        
    }
}