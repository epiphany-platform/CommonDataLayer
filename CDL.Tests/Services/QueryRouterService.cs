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
        private readonly TimeSpan[] retryPattern = new[]{
                TimeSpan.FromMilliseconds(100),
                TimeSpan.FromMilliseconds(500),
                TimeSpan.FromSeconds(1),
                TimeSpan.FromSeconds(2)
            };
        private IRestClient _client;
        private IRestRequest _request;
        private IRestResponse _response;
        private ConfigurationOptions _options;
        public QueryRouterService(IOptions<ConfigurationOptions> options)
        {
            _options = options.Value;
            _client = new RestClient(_options.CDL_QUERY_ROUTER_ADDRESS);
        }

        public QueryRouterService GetAllObjectsFromSchema(string schemaId)
        {
            _request = new RestRequest("schema", Method.GET);
            _request.AddHeader("SCHEMA_ID", schemaId);
            return this;
        }

        public QueryRouterService GetSingleObject(string schemaId, string objectId)
        {
            _request = new RestRequest($"/single/{objectId}", Method.POST);
            _request.AddHeader("SCHEMA_ID", schemaId);
            return this;
        }

        public Task<IRestResponse> ExecuteWithRetryPolicy()
        {
            return Task.FromResult(Policy.HandleResult<IRestResponse>(m => !m.IsSuccessful)
                .WaitAndRetry(retryPattern)
                .Execute(() => _client.Execute(_request)));
        }

        public Task<IRestResponse> ExecuteWithRetryPolicy(Func<IRestResponse, bool> customResultCondition)
        {
            return Task.FromResult(Policy.HandleResult<IRestResponse>(m => !m.IsSuccessful)
                .OrResult(customResultCondition)
                .WaitAndRetry(retryPattern)
                .Execute(() => _client.Execute(_request)));
        }
    }
}
