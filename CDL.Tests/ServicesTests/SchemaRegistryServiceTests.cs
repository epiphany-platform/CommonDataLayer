using System;
using System.Collections.Generic;
using System.Text.Json;
using AutoFixture;
using CDL.Tests.MessageBroker.Kafka;
using CDL.Tests.ServiceObjects.SchemaService;
using CDL.Tests.Services;
using CDL.Tests.TestDataObjects;
using MassTransit.KafkaIntegration;
using SchemaRegistry;
using Xunit;

namespace CDL.Tests.ServicesTests
{
    public class SchemaRegistryServiceTests
    {
        private SchemaRegistryService _schemaRegistryService;
        private QueryRouterService _queryService;
        private ITopicProducer<InsertObject> _kafkaProducer;
        private Fixture _fixture;

        public SchemaRegistryServiceTests(SchemaRegistryService schemaRegistryService, QueryRouterService queryService, ITopicProducer<InsertObject> kafkaProducer, Fixture fixture)
        {
            _schemaRegistryService = schemaRegistryService;
            _queryService = queryService;
            _kafkaProducer = kafkaProducer;
            _fixture = fixture;
        }

        [Fact]
        public void Heartbeat()
        {
            var results = _schemaRegistryService.Heartbeat().Result;
            Assert.NotNull(results);
            Assert.IsType<Empty>(results);
        }

        [Theory]
        [InlineData("")]
        [InlineData("MyName")]
        public void AddSchema(string schemaName)
        {
            
            var schema = _schemaRegistryService.AddSchema(schemaName, _fixture.Create<GeneralObject>().ToJSONString(), new SchemaType() { SchemaType_ = SchemaType.Types.Type.DocumentStorage }).Result;
            Assert.NotNull(schema);
            Assert.IsType<string>(schema.Id_);
            Guid.Parse(schema.Id_);
        }

        [Fact]
        public void GetSchema()
        {
            var name = _fixture.Create<string>();
            var schema = _schemaRegistryService.AddSchema(name, _fixture.Create<GeneralObject>().ToJSONString(), new SchemaType() { SchemaType_ = SchemaType.Types.Type.DocumentStorage }).Result;
            var results = _schemaRegistryService.GetFullSchema(schema.Id_).Result;

            Assert.IsType<FullSchema>(results);
            Assert.Contains(results.Metadata.Name, name);
        }

        [Fact]
        public void CheckViewAddedToSchema()
        {
            var name = _fixture.Create<string>();
            var viewName = _fixture.Create<string>();            
            var schema = _schemaRegistryService.AddSchema(name, _fixture.Create<Person>().ToJSONString(), new SchemaType() { SchemaType_ = SchemaType.Types.Type.DocumentStorage }).Result;
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
            var view = _schemaRegistryService.AddViewToSchema(schema.Id_, viewName, viewFields).Result;
            var viewDetails = _schemaRegistryService.GetView(view.Id_).Result;
                     
            Assert.NotNull(view.Id_);
            Assert.IsType<string>(view.Id_);
            Assert.NotNull(viewDetails);
            Assert.IsType<FullView>(viewDetails);
            var viewUUID = Guid.Parse(view.Id_);   
            
            var schemaWithView = _schemaRegistryService.GetFullSchema(schema.Id_).Result;
            Assert.True(schemaWithView.Views.Count == 1);
            
            var viewObject = schemaWithView.Views[0];
            Assert.Equal(viewName, viewObject.Name);
            Assert.IsType<string>(viewObject.Id);
            Assert.NotEqual("{\"Name\": \"Name\" }", viewObject.MaterializerOptions);
            Assert.NotNull(viewDetails);
            Assert.IsType<FullView>(viewDetails);
        }

        [Theory]
        [InlineData("")]
        [InlineData("{field:value}")]
        public void AddViewToSchema_wrongMaterializerOptions(string materializerOptions)
        {
            var name = _fixture.Create<string>();
            string exceptionMsg = string.Empty;
            var schema = _schemaRegistryService.AddSchema(_fixture.Create<string>(), _fixture.Create<GeneralObject>().ToJSONString(), new SchemaType() { SchemaType_ = SchemaType.Types.Type.DocumentStorage }).Result;
            
            try
            {
                _schemaRegistryService.AddViewToSchema(schema.Id_, name, new List<Simple>(), false, materializerOptions);
            }
            catch (Exception e)
            {
                exceptionMsg = e.Message;
            }

            Assert.NotNull(exceptionMsg);
            Assert.Contains("JSON error", exceptionMsg);
            Assert.Contains("StatusCode=\"Internal\"", exceptionMsg);
        }



        [Theory]
        [InlineData("")]
        [InlineData("{field:value}")]
        public void AddSchema_WrongDefinition(string definition)
        {
            string exceptionMsg = string.Empty;
            try
            {
                var results = _schemaRegistryService.AddSchema(string.Empty, definition, new SchemaType() { SchemaType_ = SchemaType.Types.Type.DocumentStorage }).Result;
            }
            catch (Exception e)
            {
                exceptionMsg = e.Message;
            }

            Assert.NotNull(exceptionMsg);
            Assert.Contains("StatusCode=\"InvalidArgument\"", exceptionMsg);
            Assert.Contains("Invalid JSON provided", exceptionMsg);

        }

        [Fact]
        public void AddObjectToSchema()
        {
            var schema = _schemaRegistryService.AddSchema(
                _fixture.Create<string>(), 
                _fixture.Create<GeneralObject>().ToJSONString(), 
                new SchemaType() { SchemaType_ = SchemaType.Types.Type.DocumentStorage 
            }).Result;
            
            var schemaObjectsBefore = _queryService.GetAllObjectsFromSchema(schema.Id_).ExecuteWithRetryPolicy().Result;
            Assert.Equal(System.Net.HttpStatusCode.OK, schemaObjectsBefore.StatusCode);
            Assert.Equal("{}", schemaObjectsBefore.Content);
            
            var objectId = Guid.NewGuid().ToString();            
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = schema.Id_,
                objectId = objectId,
                data = _fixture.Create<GeneralObject>(),
            });

            var schemaObjectsAfter = _queryService.GetAllObjectsFromSchema(schema.Id_).ExecuteWithRetryPolicy(m => m.Content.Equals("{}")).Result;
            Assert.Equal(System.Net.HttpStatusCode.OK, schemaObjectsAfter.StatusCode);
            Assert.Contains(objectId, schemaObjectsAfter.Content);
        }
    }
}