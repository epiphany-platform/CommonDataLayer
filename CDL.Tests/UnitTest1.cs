using System;
using CDL.Tests.Configuration;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.Options;
using Xunit;

namespace CDL.Tests
{
    public class UnitTest1
    {
        private readonly ConfigurationOptions _options;
        private readonly DependencyClass _d;
        private readonly DependencyClassJ _dJ;

        public UnitTest1(DependencyClass d, DependencyClassJ dj, IOptions<ConfigurationOptions> options){
            _options = options.Value;
            _d = d;
            _dJ = dj;
        }

        [Fact]
        public void Test1()
        {
            Assert.Equal("VALUE", _options.CDL_COMMAND_SERVICE_ADDRESS);
            Assert.Equal("VALUE", _options.CDL_DATA_ROUTER_ADDRESS);
            Assert.Equal("VALUE", _options.CDL_EDGE_REGISTRY_ADDRESS);
            Assert.Equal("VALUE", _options.CDL_MATERIALIZER_GENERAL_ADDRESS);
            Assert.Equal("VALUE", _options.CDL_MATERIALIZER_ONDEMAND_ADDRESS);
            Assert.Equal("VALUE", _options.CDL_QUERY_ROUTER_ADDRESS);
            Assert.Equal("VALUE", _options.CDL_QUERY_SERVICE_ADDRESS);
            Assert.Equal("VALUE", _options.CDL_SCHEMA_REGISTRY_ADDRESS);
        }
    }
}
