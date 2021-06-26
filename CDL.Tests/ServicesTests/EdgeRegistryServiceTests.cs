using System;
using System.Collections.Generic;
using AutoFixture;
using CDL.Tests.MessageBroker.Kafka;
using CDL.Tests.Services;
using CDL.Tests.TestDataObjects;
using MassTransit.KafkaIntegration;
using SchemaRegistry;
using Xunit;

namespace CDL.Tests.ServicesTests
{
    public class EdgeRegistryServiceTests
    {
        private EdgeRegistryService _edgeRegistryService;
        private SchemaRegistryService _schemaRegistryService;
        private ITopicProducer<InsertObject> _kafkaProducer;
        private Fixture _fixture;
        
        public EdgeRegistryServiceTests(SchemaRegistryService schemaRegistryService, EdgeRegistryService edgeRegistryService, ITopicProducer<InsertObject> kafkaProducer, Fixture fixture)
        {
            _edgeRegistryService = edgeRegistryService;
            _schemaRegistryService = schemaRegistryService;
            _kafkaProducer = kafkaProducer;
            _fixture = fixture;
        }

        [Fact]
        public void AddRelation()
        {
            var parentName = _fixture.Create<string>();
            var childName = _fixture.Create<string>();
            var parentSchema = _schemaRegistryService.AddSchema(parentName, _fixture.Create<Person>().ToJSONString(), new SchemaType() { SchemaType_ = SchemaType.Types.Type.DocumentStorage }).Result;
            var childSchema = _schemaRegistryService.AddSchema(childName, _fixture.Create<Car>().ToJSONString(), new SchemaType() { SchemaType_ = SchemaType.Types.Type.DocumentStorage }).Result;

            var results = _edgeRegistryService.AddRelation(childSchema.Id_, parentSchema.Id_).Result;

            Assert.True(results.HasRelationId_);
            Assert.Matches("(\\{){0,1}[0-9a-fA-F]{8}\\-[0-9a-fA-F]{4}\\-[0-9a-fA-F]{4}\\-[0-9a-fA-F]{4}\\-[0-9a-fA-F]{12}(\\}){0,1}", results.RelationId_);
        }

        [Fact]
        public void ListRelations()
        {          
            var parentSchemaName = _fixture.Create<string>();
            var childSchemaName = _fixture.Create<string>();
            var parentSchema = _schemaRegistryService.AddSchema(parentSchemaName, _fixture.Create<Person>().ToJSONString(), new SchemaType() { SchemaType_ = SchemaType.Types.Type.DocumentStorage }).Result;
            var childSchema = _schemaRegistryService.AddSchema(childSchemaName, _fixture.Create<Car>().ToJSONString(), new SchemaType() { SchemaType_ = SchemaType.Types.Type.DocumentStorage }).Result;
            var relation = _edgeRegistryService.AddRelation(childSchema.Id_, parentSchema.Id_).Result;
            var a = _edgeRegistryService.ListRelations();
                   
        }

        [Fact]
        public void GetEdge(){
            var objectIdForParentSchema= Guid.NewGuid().ToString(); 
            var objectIdForChildSchema = Guid.NewGuid().ToString();                
            var parentSchemaName = _fixture.Create<string>();
            var childSchemaName = _fixture.Create<string>();
            var parentSchema = _schemaRegistryService.AddSchema(parentSchemaName, _fixture.Create<Person>().ToJSONString(), new SchemaType() { SchemaType_ = SchemaType.Types.Type.DocumentStorage }).Result;
            var childSchema = _schemaRegistryService.AddSchema(childSchemaName, _fixture.Create<Car>().ToJSONString(), new SchemaType() { SchemaType_ = SchemaType.Types.Type.DocumentStorage }).Result;
            var relation = _edgeRegistryService.AddRelation(childSchema.Id_, parentSchema.Id_).Result;
                   
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = parentSchema.Id_,
                objectId = objectIdForParentSchema,
                data = _fixture.Create<GeneralObject>(),
            });
            var idChild = Guid.NewGuid().ToString();
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = childSchema.Id_,
                objectId = objectIdForChildSchema,
                data = _fixture.Create<Car>(),
            });
            var listOfObjects = new List<string>();
            listOfObjects.Add(idChild);


            
            _kafkaProducer.Produce(new InsertEdgeObject()
            {
                relation_id = relation.RelationId_,
                parent_object_id = parentSchema.Id_, 
                child_object_ids = listOfObjects
            });
            var tmp1 = _edgeRegistryService.AddEdges().Result;
            var tmp = _edgeRegistryService.GetEdge(parentSchema.Id_, relation.RelationId_).Result;
        }

        [Fact]
        public void Heartbeat()
        {
            var results = _edgeRegistryService.Heartbeat().Result;
            Assert.IsType<Empty>(results);
        }
    }
}