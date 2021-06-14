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
        public string CDL_KAFKA_ADDRESS { get ;set; }
    }
}