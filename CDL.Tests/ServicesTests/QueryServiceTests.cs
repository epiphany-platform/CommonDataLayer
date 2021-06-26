using System;
using CDL.Tests.Services;
using Xunit;

namespace CDL.Tests.ServicesTests
{
    public class QueryServiceTests 
    {
        private QueryRouterService _queryService;
        public QueryServiceTests(QueryRouterService queryService)
        {
            _queryService = queryService;
        }

        [Theory]
        [InlineData("")]
        [InlineData("123")]
        public void GetSchema_WrongSchemaId(string valueToTest)
        {
            var results = _queryService.GetAllObjectsFromSchema(valueToTest).ExecuteWithRetryPolicy().Result;

            Assert.Equal(System.Net.HttpStatusCode.BadRequest, results.StatusCode);
            Assert.Contains("Invalid request header", results.Content);
        }

        [Fact]
        public void GetSchema_SchemaNotExist()
        {
            var results = _queryService.GetAllObjectsFromSchema(Guid.NewGuid().ToString()).ExecuteWithRetryPolicy().Result;

            Assert.Equal(System.Net.HttpStatusCode.InternalServerError, results.StatusCode);
            Assert.Contains("No schema found", results.Content);
        }
    }
}
