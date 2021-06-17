namespace CDL.Tests.Configuration
{
    public class ConfigurationOptions
    {
        public string CDL_COMMAND_SERVICE_ADDRESS { get; set; } 
        public string CDL_DATA_ROUTER_ADDRESS { get; set; } 
        public string CDL_EDGE_REGISTRY_ADDRESS { get; set; } 
        public string CDL_MATERIALIZER_GENERAL_ADDRESS { get; set; } 
        public string CDL_MATERIALIZER_ONDEMAND_ADDRESS { get; set; } 
        public string CDL_QUERY_ROUTER_ADDRESS { get; set; } 
        public string CDL_QUERY_SERVICE_ADDRESS { get; set; } 
        public string CDL_SCHEMA_REGISTRY_ADDRESS { get; set; }  
        public string CDL_KAFKA_BROKER { get; set; }
        public string CDL_KAFKA_DATA_ROUTER_GROUP { get; set; }
        public string CDL_KAFKA_DATA_INPUT_TOPIC { get; set; }
        public string CDL_KAFKA_EDGE_REGISTRY_GROUP { get; set; }
        public string CDL_KAFKA_EDGE_INPUT_TOPIC { get; set; }
        public string CDL_KAFKA_OBJECT_BUILDER_GROUP { get; set; }
        public string CDL_KAFKA_SCHEMA_REGISTRY_GROUP { get; set; }
        public string CDL_KAFKA_OBJECT_BUILDER_TOPIC { get; set; }
        public string CDL_DOCUMENT_ORDERED_DATA { get; set; }
        public string CDL_DOCUMENT_UNORDERED_DATA { get; set; }
        public string CDL_TIMESERIES_ORDERED_DATA { get; set; }
        public string CDL_TIMESERIES_UNORDERED_DATA { get; set; }
        public string CDL_SCHEMA_REGISTRY_DESTINATION { get; set; }
    }
}