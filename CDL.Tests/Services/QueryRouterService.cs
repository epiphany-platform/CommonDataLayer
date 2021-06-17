using System;
using System.Threading.Tasks;
using CDL.Tests.Configuration;
using Microsoft.Extensions.Options;
using Polly;
using RestSharp;

namespace CDL.Tests.Services
{
    public class QueryRouterService
    {
        private IRestClient _restClient;
        private ConfigurationOptions _options;
        public QueryRouterService(IOptions<ConfigurationOptions> options)
        {
            _options = options.Value;
            _restClient = new RestClient(_options.CDL_QUERY_ROUTER_ADDRESS);
        }

        public Task<IRestResponse> GetAllObjectsFromSchema(string schemaId)
        {
            var request = new RestRequest("schema", Method.GET);
            request.AddHeader("SCHEMA_ID", schemaId);
            return Task.FromResult(ExecuteWithPolly(request));
        }

        public Task<IRestResponse> GetSingleObject(string schemaId, string objectId)
        {
            var request = new RestRequest($"/single/{objectId}", Method.POST);
            request.AddHeader("SCHEMA_ID", schemaId);
            return Task.FromResult(ExecuteWithPolly(request));
        }

        private IRestResponse ExecuteWithPolly(IRestRequest request)
        {
            return Policy.HandleResult<IRestResponse>(m => !m.IsSuccessful)
                .WaitAndRetry(new[]{
                    TimeSpan.FromMilliseconds(500),
                    TimeSpan.FromSeconds(1),
                    TimeSpan.FromSeconds(2)
                })
                .Execute(() => _restClient.Execute(request));
        }
    }
}
