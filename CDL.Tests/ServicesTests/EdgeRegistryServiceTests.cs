using System;
using System.Collections.Generic;
using AutoFixture;
using CDL.Tests.MessageBroker.Kafka;
using CDL.Tests.Services;
using CDL.Tests.TestDataObjects;
using EdgeRegistry;
using MassTransit.KafkaIntegration;
using SchemaRegistry;
using Xunit;
using Empty = EdgeRegistry.Empty;

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
            var relationListBefore = _edgeRegistryService.ListRelations().Result;
            var parentSchema = _schemaRegistryService.AddSchema(_fixture.Create<string>(), _fixture.Create<Person>().ToJSONString(), new SchemaType() { SchemaType_ = SchemaType.Types.Type.DocumentStorage }).Result;
            var childSchema = _schemaRegistryService.AddSchema(_fixture.Create<string>(), _fixture.Create<Car>().ToJSONString(), new SchemaType() { SchemaType_ = SchemaType.Types.Type.DocumentStorage }).Result;
            var relation = _edgeRegistryService.AddRelation(childSchema.Id_, parentSchema.Id_).Result;
            var relationListAfter = _edgeRegistryService.ListRelations().Result;

            Assert.True(relationListBefore.Items.Count < relationListAfter.Items.Count);
            
            var item = relationListAfter.Items[relationListAfter.Items.Count - 1];            
            Assert.Equal(parentSchema.Id_, item.ParentSchemaId);
            Assert.True(item.HasChildSchemaId);
            Assert.Equal(childSchema.Id_, item.ChildSchemaId);                   
        }

        [Fact]
        public void GetSchemaByRelation()
        {
            var parentSchema = _schemaRegistryService.AddSchema(_fixture.Create<string>(), _fixture.Create<Person>().ToJSONString(), new SchemaType() { SchemaType_ = SchemaType.Types.Type.DocumentStorage }).Result;
            var childSchema = _schemaRegistryService.AddSchema(_fixture.Create<string>(), _fixture.Create<Car>().ToJSONString(), new SchemaType() { SchemaType_ = SchemaType.Types.Type.DocumentStorage }).Result;
            var relation = _edgeRegistryService.AddRelation(childSchema.Id_, parentSchema.Id_).Result;
            var schemaFromRelation = _edgeRegistryService.GetSchemaByRelation(relation.RelationId_).Result;

            Assert.Equal(parentSchema.Id_, schemaFromRelation.ParentSchemaId);
            Assert.True(schemaFromRelation.HasChildSchemaId);
            Assert.Equal(childSchema.Id_, schemaFromRelation.ChildSchemaId);  
        }

        [Fact]
        public void ResolveTree()
        {
            var objectIdForParentSchema= Guid.NewGuid().ToString(); 
            var objectIdForChildSchema = Guid.NewGuid().ToString();                
            var parentSchemaName = _fixture.Create<string>();
            var childSchemaName = _fixture.Create<string>();
            var parentSchema = _schemaRegistryService.AddSchema(parentSchemaName, _fixture.Create<Person>().ToJSONString(), new SchemaType() { SchemaType_ = SchemaType.Types.Type.DocumentStorage }).Result;
            var childSchema = _schemaRegistryService.AddSchema(childSchemaName, _fixture.Create<Car>().ToJSONString(), new SchemaType() { SchemaType_ = SchemaType.Types.Type.DocumentStorage }).Result;
                        
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = parentSchema.Id_,
                objectId = objectIdForParentSchema,
                data = _fixture.Create<Person>(),
            });
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = childSchema.Id_,
                objectId = objectIdForChildSchema,
                data = _fixture.Create<Car>(),
            });       

            var relation = _edgeRegistryService.AddRelation(objectIdForChildSchema, objectIdForParentSchema).Result;
            var newEdge = new Edge()
                {
                    RelationId = relation.RelationId_,
                    ParentObjectId = objectIdForParentSchema,                  
                };
            newEdge.ChildObjectIds.Add(objectIdForChildSchema);
            _edgeRegistryService.AddEdges(newEdge);    
            var tree = _edgeRegistryService.ResolveTree(relation.RelationId_).Result;


            Assert.True(tree.Objects.Count == 1);
            var treeObject = tree.Objects[0];
            Assert.True(treeObject.HasObjectId);
            Assert.True(treeObject.RelationId.Equals(relation.RelationId_));
            Assert.True(treeObject.Relation.ParentSchemaId.Equals(objectIdForParentSchema));
            Assert.True(treeObject.Relation.ChildSchemaId.Equals(objectIdForChildSchema));
        }

        [Fact]
        public void GetEdge(){
            var objectIdForParentSchema= Guid.NewGuid().ToString(); 
            var objectIdForChildSchema = Guid.NewGuid().ToString();                
            var parentSchemaName = _fixture.Create<string>();
            var childSchemaName = _fixture.Create<string>();
            var parentSchema = _schemaRegistryService.AddSchema(parentSchemaName, _fixture.Create<Person>().ToJSONString(), new SchemaType() { SchemaType_ = SchemaType.Types.Type.DocumentStorage }).Result;
            var childSchema = _schemaRegistryService.AddSchema(childSchemaName, _fixture.Create<Car>().ToJSONString(), new SchemaType() { SchemaType_ = SchemaType.Types.Type.DocumentStorage }).Result;
                        
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = parentSchema.Id_,
                objectId = objectIdForParentSchema,
                data = _fixture.Create<Person>(),
            });
            _kafkaProducer.Produce(new InsertObject()
            {
                schemaId = childSchema.Id_,
                objectId = objectIdForChildSchema,
                data = _fixture.Create<Car>(),
            });       

            var relation = _edgeRegistryService.AddRelation(objectIdForChildSchema, objectIdForParentSchema).Result;
            var newEdge = new Edge()
                {
                    RelationId = relation.RelationId_,
                    ParentObjectId = objectIdForParentSchema,                  
                };
            newEdge.ChildObjectIds.Add(objectIdForChildSchema);
            _edgeRegistryService.AddEdges(newEdge);    

            var edgeFromService = _edgeRegistryService.GetEdge(objectIdForParentSchema, relation.RelationId_).Result;

            Assert.True(edgeFromService.HasRelationId);
            Assert.True(edgeFromService.HasParentObjectId);
            Assert.True(edgeFromService.RelationId.Equals(relation.RelationId_));
            Assert.True(edgeFromService.ParentObjectId.Equals(objectIdForParentSchema));
            Assert.True(edgeFromService.ChildObjectIds.Count == 1);
            Assert.True(edgeFromService.ChildObjectIds[0].Equals(objectIdForChildSchema));
        }

        [Fact]
        public void GetSchemaRelations()
        {          
            var parentSchemaName = _fixture.Create<string>();
            var childSchemaName = _fixture.Create<string>();
            var parentSchema = _schemaRegistryService.AddSchema(parentSchemaName, _fixture.Create<Person>().ToJSONString(), new SchemaType() { SchemaType_ = SchemaType.Types.Type.DocumentStorage }).Result;
            var childSchema = _schemaRegistryService.AddSchema(childSchemaName, _fixture.Create<Car>().ToJSONString(), new SchemaType() { SchemaType_ = SchemaType.Types.Type.DocumentStorage }).Result;       
            var relation = _edgeRegistryService.AddRelation(childSchema.Id_, parentSchema.Id_).Result;
            var schemaRelations = _edgeRegistryService.GetSchemaRelations(parentSchema.Id_).Result;

            Assert.True(schemaRelations.Items.Count == 1);
            var item = schemaRelations.Items[0];

            Assert.Equal(parentSchema.Id_, item.ParentSchemaId);
            Assert.True(item.HasChildSchemaId);
            Assert.Equal(childSchema.Id_, item.ChildSchemaId);
        }

        [Fact]
        public void Heartbeat()
        {
            var results = _edgeRegistryService.Heartbeat().Result;
            Assert.IsType<Empty>(results);
        }
    }
}