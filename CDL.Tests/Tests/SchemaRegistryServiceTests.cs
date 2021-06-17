using System;
using AutoFixture;
using CDL.Tests.GrpcServices;
using SchemaRegistry;
using Xunit;

namespace CDL.Tests.Tests
{
    public class SchemaRegistryServiceTests
    {
        private SchemaRegistryService _schemaRegistryService;
        private Fixture _fixture;
        public SchemaRegistryServiceTests(SchemaRegistryService schemaRegistryService, Fixture fixture)
        {
            _schemaRegistryService = schemaRegistryService;
            _fixture = fixture;
        }
        [Fact]
        public void ServicePing()
        {
            var results = _schemaRegistryService.Ping().Result;
            Assert.NotNull(results);
            Assert.IsType<Empty>(results);
        }

        [Theory]
        [InlineData("")]
        [InlineData("MyName")]
        public void AddSchema(string schemaName)
        {
            Guid schemaUUID;
            var schema = _schemaRegistryService.AddSchema(schemaName, "{\"name\": \"test\"}", new SchemaType() { SchemaType_ = SchemaType.Types.Type.DocumentStorage }).Result;
            Guid.TryParse(schema.Id_, out schemaUUID);

            Assert.NotNull(schema);
            Assert.IsType<string>(schema.Id_);
            Assert.NotEqual("00000000-0000-0000-0000-000000000000", schemaUUID.ToString());
        }

        [Fact]
        public void GetSchema()
        {
            var name = _fixture.Create<string>();
            var schema = _schemaRegistryService.AddSchema(name, "{\"name\": \"test\"}", new SchemaType() { SchemaType_ = SchemaType.Types.Type.DocumentStorage }).Result;
            var results = _schemaRegistryService.GetFullSchema(schema.Id_).Result;

            Assert.NotNull(results);
            Assert.IsType<FullSchema>(results);
            Assert.Contains(results.Metadata.Name, name);
        }

        [Fact]
        public void CheckViewAddedToSchema()
        {
            var name = _fixture.Create<string>();
            Guid viewUUID;
            var schema = _schemaRegistryService.AddSchema(name, "{\"name\": \"test\"}", new SchemaType() { SchemaType_ = SchemaType.Types.Type.DocumentStorage }).Result;
            var view = _schemaRegistryService.AddViewToSchema(schema.Id_, _fixture.Create<string>(), "{\"Name\": \"Name\" }").Result;
            var viewDetails = _schemaRegistryService.GetView(view.Id_).Result;

            Guid.TryParse(view.Id_, out viewUUID);
            Assert.NotNull(view.Id_);
            Assert.IsType<string>(view.Id_);
            Assert.NotEqual("00000000-0000-0000-0000-000000000000", viewUUID.ToString());
            Assert.NotNull(viewDetails);
            Assert.IsType<FullView>(viewDetails);
            Assert.NotEqual(name, viewDetails.Name);
        }

        [Theory]
        [InlineData("")]
        [InlineData("{field:value}")]
        public void AddViewToSchema_wrongMaterializerOptions(string materializerOptions)
        {
            var name = _fixture.Create<string>();
            string exceptionMsg = string.Empty;
            var schema = _schemaRegistryService.AddSchema(_fixture.Create<string>(), "{\"name\": \"test\"}", new SchemaType() { SchemaType_ = SchemaType.Types.Type.DocumentStorage }).Result;
            
            try
            {
                _schemaRegistryService.AddViewToSchema(schema.Id_, name, materializerOptions);
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

        // [Fact]
        // public void GetObjectsAndObjectDetailsFromSchema()
        // {
        //     var schemaId = _schemaRegistryService.AddSchema(_fixture.Create<string>(), "{\"name\": \"test\"}", new SchemaType() { SchemaType_ = SchemaType.Types.Type.DocumentStorage }).Result;
        //     var schemaObjectsBefore = _queryRouterService.GetAllObjectsFromSchema(schemaId).Result;
        //     var objectId = Guid.NewGuid().ToString();
        //     var payload = new InsertDataObject()
        //     {
        //         schemaId = schemaId,
        //         objectId = objectId,
        //         data = _testData
        //     };
        //     _dataRouterService.Messege("NULL", JsonSerializer.Serialize(payload));
        //     var schemaObjectsAfter = _queryRouterService.GetAllObjectsFromSchema(schemaId).Result;
        //     var objectDetails = _queryRouterService.GetSingleObject(schemaId, objectId).Result;

        //     Assert.NotNull(schemaId);
        //     Assert.NotNull(schemaObjectsBefore);
        //     Assert.NotNull(schemaObjectsAfter);
        //     Assert.NotNull(objectDetails);

        //     var testDataFromResponse = JsonSerializer.Deserialize<IList<GeneralObject>>(schemaObjectsBefore.Content);
        //     var testDataFromResponseAfter = JsonSerializer.Deserialize<IList<GeneralObject>>(schemaObjectsAfter.Content);
        // }
    }
}