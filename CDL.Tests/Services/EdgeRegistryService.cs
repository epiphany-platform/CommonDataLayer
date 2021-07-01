using CDL.Tests.Configuration;
using EdgeRegistry;
using Microsoft.Extensions.Options;
using System.Threading.Tasks;
using static EdgeRegistry.EdgeRegistry;

namespace CDL.Tests.Services
{
    public class EdgeRegistryService
    {
        private EdgeRegistryClient _client;
        private ConfigurationOptions _options;
        
        public EdgeRegistryService(IOptions<ConfigurationOptions> options, EdgeRegistryClient client)
        {            
            _options = options.Value;
            _client = client;
        }

        public Task<Empty> AddEdges(Edge edgeItem)
        {
            var relation = new ObjectRelations();
            relation.Relations.Add(edgeItem);
            var response = _client.AddEdges(relation);
            return Task.FromResult(response);
        }

        public async Task<Empty> AddEdgesAsync()
        {
            var response = await _client.AddEdgesAsync(new ObjectRelations());
            return Task.FromResult(response).Result;
        }

        public Task<RelationId> AddRelation(string childSchemaId, string parentSchemaId)
        {
            var schemaRelation = new AddSchemaRelation() 
            { 
                ParentSchemaId = parentSchemaId, 
                ChildSchemaId = childSchemaId,                
            };
            var response = _client.AddRelation(schemaRelation);
            return Task.FromResult(response);
        }

        public async Task<RelationId> AddRelationAsync(string childSchemaId, string parentSchemaId)
        {
            var schemaRelation = new AddSchemaRelation()
            {
                ChildSchemaId = childSchemaId,
                ParentSchemaId = parentSchemaId
            };
            var response = await _client.AddRelationAsync(schemaRelation);
            return Task.FromResult(response).Result;
        }

        public Task<Edge> GetEdge(string parentObjectId, string relationId)
        {
            var relationIdQuery = new RelationIdQuery()
            {
                ParentObjectId = parentObjectId,
                RelationId = relationId
            };
            var response = _client.GetEdge(relationIdQuery);
            return Task.FromResult(response);
        }

        public async Task<Edge> GetEdgeAsync(string parentObjectId, string relationId)
        {
            var relationIdQuery = new RelationIdQuery()
            {
                ParentObjectId = parentObjectId,
                RelationId = relationId
            };
            var response = await _client.GetEdgeAsync(relationIdQuery);
            return Task.FromResult(response).Result;
        }

        public Task<ObjectRelations> GetEdges(string objectId)
        {
            var objectIdQuery = new ObjectIdQuery()
            {
                ObjectId = objectId
            };
            var response = _client.GetEdges(objectIdQuery);
            return Task.FromResult(response);
        }

        public async Task<ObjectRelations> GetEdgesAsync(string objectId)
        {
            var objectIdQuery = new ObjectIdQuery()
            {
                ObjectId = objectId
            };
            var response = await _client.GetEdgesAsync(objectIdQuery);
            return Task.FromResult(response).Result;
        }

        public Task<RelationResponse> GetRelation(string parentSchemaId, string relationId)
        {
            var relationQuery = new RelationQuery()
            {
                ParentSchemaId = parentSchemaId,
                RelationId = relationId
            };
            var response = _client.GetRelation(relationQuery);
            return Task.FromResult(response);
        }

        public async Task<RelationResponse> GetRelationAsync(string parentSchemaId, string relationId)
        {
            var relationQuery = new RelationQuery()
            {
                ParentSchemaId = parentSchemaId,
                RelationId = relationId
            };
            var response = await _client.GetRelationAsync(relationQuery);
            return Task.FromResult(response).Result;
        }

        public Task<RelationList> GetSchemaRelations(string schemaIdentity)
        {
            var schemaId = new SchemaId()
            {
                SchemaId_ = schemaIdentity
            };
            var response = _client.GetSchemaRelations(schemaId);
            return Task.FromResult(response);
        }

        public async Task<RelationList> GetSchemaRelationsAsync(string schemaIdentity)
        {
            var schemaId = new SchemaId()
            {
                SchemaId_ = schemaIdentity
            };
            var response = await _client.GetSchemaRelationsAsync(schemaId);
            return Task.FromResult(response).Result;
        }

        public Task<SchemaRelation> GetSchemaByRelation(string relationId)
        {
            var relationIdObject = new RelationId()
            {
                RelationId_ = relationId
            };
            var response = _client.GetSchemaByRelation(relationIdObject);
            return Task.FromResult(response);
        }

        public Task<Empty> Heartbeat()
        {
            var response = _client.Heartbeat(new Empty());
            return Task.FromResult(response);
        }

        public async Task<Empty> HeartbeatAsync()
        {
            var response = await _client.HeartbeatAsync(new Empty());
            return Task.FromResult(response).Result;
        }

        public Task<RelationList> ListRelations()
        {
            var response = _client.ListRelations(new Empty());
            return Task.FromResult(response);
        }

        public async Task<RelationList> ListRelationsAsync()
        {
            var response = await _client.ListRelationsAsync(new Empty());
            return Task.FromResult(response).Result;
        }

        public Task<TreeResponse> ResolveTree(string relationId)
        {
            var treeQuery = new TreeQuery()
            {
                RelationId = relationId
            };
            var response = _client.ResolveTree(treeQuery);
            return Task.FromResult(response);
        }

        public async Task<TreeResponse> ResolveTreeAsync(string relationId)
        {
            var treeQuery = new TreeQuery()
            {
                RelationId = relationId
            };
            var response = await _client.ResolveTreeAsync(treeQuery);
            return Task.FromResult(response).Result;
        }

        public Task<Empty> ValidateRelation(string relationId)
        {
            var validateRelationQuery = new ValidateRelationQuery()
            {
                RelationId = relationId
            };
            var response = _client.ValidateRelation(validateRelationQuery);
            return Task.FromResult(response);
        }

        public async Task<Empty> ValidateRelationAsync(string relationId)
        {
            var validateRelationQuery = new ValidateRelationQuery()
            {
                RelationId = relationId
            };
            var response = await _client.ValidateRelationAsync(validateRelationQuery);
            return Task.FromResult(response).Result;
        }
    }
}
