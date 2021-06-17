using AutoFixture;
using CDL.Tests.Services;
using CDL.Tests.TestDataObjects;
using SchemaRegistry;
using Xunit;

namespace CDL.Tests.ServicesTests
{
    public class EdgeRegistryServiceTests
    {
        private EdgeRegistryService _edgeRegistryService;
        private SchemaRegistryService _schemaRegistryService;
        private Fixture _fixture;
        
        public EdgeRegistryServiceTests(SchemaRegistryService schemaRegistryService, EdgeRegistryService edgeRegistryService, Fixture fixture)
        {
            _edgeRegistryService = edgeRegistryService;
            _schemaRegistryService = schemaRegistryService;
            _fixture = fixture;
        }

        [Fact]
        public void AddRelation()
        {
            var parentName = _fixture.Create<string>();
            var childName = _fixture.Create<string>();
            var parentSchema = _schemaRegistryService.AddSchema(parentName, _fixture.Create<Person>().ToJSONString(), new SchemaType() { SchemaType_ = SchemaType.Types.Type.DocumentStorage }).Result;
            var childSchema = _schemaRegistryService.AddSchema(childName, _fixture.Create<Car>().ToJSONString(), new SchemaType() { SchemaType_ = SchemaType.Types.Type.DocumentStorage }).Result;

            var results = _edgeRegistryService.AddRelation(childSchema.Id_, parentSchema.Id_);

            Assert.NotNull(results);
        }
    }
}