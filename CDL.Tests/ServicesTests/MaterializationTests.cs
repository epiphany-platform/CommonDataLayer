using System;
using System.Collections.Generic;
using AutoFixture;
using CDL.Tests.MessageBroker.Kafka;
using CDL.Tests.ServiceObjects.SchemaService;
using CDL.Tests.Services;
using CDL.Tests.TestDataObjects;
using Grpc.Core;
using MassTransit.KafkaIntegration;
using SchemaRegistry;
using Xunit;

namespace CDL.Tests.ServicesTests
{
    public class MaterializationTests
    {
        private OnDemandMaterializerService _onDemandMaterializerService;
        private SchemaRegistryService _schemaRegistryService;
        private QueryRouterService _queryService;
        private ITopicProducer<InsertObject> _kafkaProducer;
        private Fixture _fixture;

        public MaterializationTests(
            OnDemandMaterializerService onDemandMaterializerService,
            SchemaRegistryService schemaRegistryService, 
            QueryRouterService queryService, 
            ITopicProducer<InsertObject> kafkaProducer, 
            Fixture fixture)
        {
            _onDemandMaterializerService = onDemandMaterializerService;
            _schemaRegistryService = schemaRegistryService;
            _queryService = queryService;
            _kafkaProducer = kafkaProducer;
            _fixture = fixture;
        }

        [Fact]
        public void OnDemandMaterializerServiceHeartbeat()
        {
            var results = _onDemandMaterializerService.Heartbeat().Result;
            Assert.NotNull(results);
            Assert.IsType<Empty>(results);
        }

        [Fact]
        public async void TestName()
        {
            var name = _fixture.Create<string>();
            var objectId_a = Guid.NewGuid().ToString(); 
            var objectId_b = Guid.NewGuid().ToString(); 
            var viewName = _fixture.Create<string>();            
            var schema = _schemaRegistryService.AddSchema(name, _fixture.Create<GeneralObject>().ToJSONString(), new SchemaType() { SchemaType_ = SchemaType.Types.Type.DocumentStorage }).Result;
            var viewFields = new List<Simple>();
            viewFields.Add(new Simple()
                {
                    simple = new SimpleItem()
                    {
                        field_name = "FirstName",
                        field_type = "String" 
                    }
                });
            viewFields.Add(new Simple()
                {
                    simple = new SimpleItem()
                    {
                        field_name = "LastName",
                        field_type = "String" 
                    }
                });
            var view = _schemaRegistryService.AddViewToSchema(schema.Id_, viewName, viewFields, true).Result;
            var viewDetails = _schemaRegistryService.GetView(view.Id_).Result;
            var schemaWithView = _schemaRegistryService.GetFullSchema(schema.Id_).Result;
            Assert.Equal(1, schemaWithView.Views.Count);
            
            await _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = schema.Id_,
                objectId = objectId_a,
                data = _fixture.Create<Person>(),
            });

            await _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = schema.Id_,
                objectId = objectId_b,
                data = _fixture.Create<Person>(),
            });

            var res = _onDemandMaterializerService.Materialize(view.Id_, schema.Id_, new List<string>(){
                objectId_a,
                objectId_b
            });

            while (await res.ResponseStream.MoveNext())
            {
                Console.WriteLine(res.ResponseStream.Current.Fields);
                //TODO Add Assertions
            }
        }

        
    }
}