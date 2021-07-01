using System.Collections.Generic;
using System.Text.Json;
using System.Threading.Tasks;
using CDL.Tests.Configuration;
using CDL.Tests.ServiceObjects.SchemaService;
using Google.Protobuf;
using Grpc.Core;
using Microsoft.Extensions.Options;
using SchemaRegistry;
using static SchemaRegistry.SchemaRegistry;

namespace CDL.Tests.Services
{
    public class SchemaRegistryService
    {
        private SchemaRegistryClient _client;
        private ConfigurationOptions _options;
        public SchemaRegistryService(SchemaRegistryClient client, IOptions<ConfigurationOptions> options)
        {
            _client = client;
            _options = options.Value;
        }

        public Task<Id> AddSchema(string name, string definition, SchemaType type)
        {
            var schema = new NewSchema()
            {
                Metadata = new SchemaMetadata()
                {
                    InsertDestination = _options.CDL_SCHEMA_REGISTRY_DESTINATION,
                    Name = name,
                    QueryAddress = _options.CDL_QUERY_SERVICE_ADDRESS,
                    SchemaType = type
                },
                Definition = ByteString.CopyFromUtf8(definition)
            };
            
            return Task.FromResult(_client.AddSchema(schema));
        }

        public Task<Empty> AddSchemaVersion(string schemaId, string data, string definition, string version)
        {
            var newSchemaVersion = new NewSchemaVersion()
            {
                Id = schemaId,
                Definition = new SchemaDefinition()
                {
                    Definition = ByteString.CopyFromUtf8(definition),
                    Version = version
                }
            };

            return Task.FromResult(_client.AddSchemaVersion(newSchemaVersion));
        }

        public Task<Empty> UpdateSchema(string schemaId, string name, string definition, SchemaType type)
        {
            var newUpdateSchema = new SchemaMetadataUpdate()
            {
                Id = schemaId,
                Patch = new SchemaMetadataPatch()
                {
                    InsertDestination = _options.CDL_COMMAND_SERVICE_ADDRESS,
                    QueryAddress = _options.CDL_QUERY_SERVICE_ADDRESS,
                    Name = name,
                    SchemaType = type
                }
            };

            return Task.FromResult(_client.UpdateSchema(newUpdateSchema));
        }

        public Task<Id> AddViewToSchema(string schemaId, string name, IList<Simple> materializerFields, bool materializerOnDemand = false, string materializerOptions = "{}")
        {
            var view = new NewView()
            {
                BaseSchemaId = schemaId,
                Name = name,
                MaterializerAddress = materializerOnDemand ? string.Empty : _options.CDL_MATERIALIZER_GENERAL_ADDRESS,
                MaterializerOptions = materializerOptions,
            };

            foreach (var field in materializerFields)
            {
                view.Fields.Add(field.simple.field_name, JsonSerializer.Serialize(field));
            }            
            
            return Task.FromResult(_client.AddViewToSchema(view));
        }

        public Task<Empty> UpdateView(string schemaId, string name, string materializerOptions, bool updateFields)
        {
            var newView = new ViewUpdate()
            {
                Id = schemaId,
                MaterializerAddress = _options.CDL_MATERIALIZER_GENERAL_ADDRESS,
                MaterializerOptions = materializerOptions,
                Name = name,
                UpdateFields = updateFields
            };
            return Task.FromResult(_client.UpdateView(newView));
        }

        public Task<SchemaMetadata> GetSchemaMetadata(string schemaId)
        {
            return Task.FromResult(_client.GetSchemaMetadata(new Id()
            {
                Id_ = schemaId
            }));
        }

        public Task<SchemaVersions> GetSchemaVersions(string schemaId)
        {
            return Task.FromResult(_client.GetSchemaVersions(new Id()
            {
                Id_ = schemaId
            }));
        }

        public Task<SchemaDefinition> GetSchemaDefinition(string schemaId, string versionReq)
        {
            return Task.FromResult(_client.GetSchemaDefinition(new VersionedId()
            {
                Id = schemaId,
                VersionReq = versionReq
            }));
        }

        public async Task<AsyncUnaryCall<SchemaMetadata>> GetSchemaMetadataAsync(string schemaId)
        {
            return await Task.FromResult(_client.GetSchemaMetadataAsync(new Id()
            {
                Id_ = schemaId
            }));
        }

        public Task<FullSchema> GetFullSchema(string schemaId)
        {
            return Task.FromResult(_client.GetFullSchema(new Id()
            {
                Id_ = schemaId
            }));
        }

        public Task<FullView> GetView(string viewId)
        {
            return Task.FromResult(_client.GetView(new Id()
            {
                Id_ = viewId
            }));
        }

        public Task<Schemas> GetAllSchemas()
        {
            return Task.FromResult(_client.GetAllSchemas(new Empty()));
        }

        public Task<FullSchemas> GetAllFullSchemas()
        {
            return Task.FromResult(_client.GetAllFullSchemas(new Empty()));
        }

        public Task<AsyncUnaryCall<FullSchemas>> GetAllFullSchemasAsync()
        {
            return Task.FromResult(_client.GetAllFullSchemasAsync(new Empty()));
        }

        public Task<AsyncUnaryCall<Id>> AddViewToSchemaAsync(string schemaId, string name, string materializerOptions)
        {
            var view = new NewView()
            {
                BaseSchemaId = schemaId,
                Name = name,
                MaterializerAddress = _options.CDL_MATERIALIZER_ONDEMAND_ADDRESS,
                MaterializerOptions = materializerOptions,
            };
            var response = _client.AddViewToSchemaAsync(view);
            schemaId = response.ToString();
            return Task.FromResult(response);
        }

        public Task<Errors> ValidateValue(string objectId, string valueToCheck, string versionReq)
        {
            var valueToValidate = new ValueToValidate()
            {
                SchemaId = new VersionedId()
                {
                    Id = objectId,
                    VersionReq = versionReq
                },
                Value = ByteString.CopyFromUtf8(valueToCheck)
            };
            return Task.FromResult(_client.ValidateValue(valueToValidate));
        }

        public Task<AsyncServerStreamingCall<Schema>> WatchAllSchemaUpdates(CallOptions options)
        {
            return Task.FromResult(_client.WatchAllSchemaUpdates(new Empty(), options));
        }
        public Task<Empty> Heartbeat()
        {
            return Task.FromResult(_client.Heartbeat(new Empty()));
        }
    }
}