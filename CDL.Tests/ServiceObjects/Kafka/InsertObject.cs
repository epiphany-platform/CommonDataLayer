namespace CDL.Tests.MessageBroker.Kafka
{
    public class InsertObject
    {
        public string schemaId { get; set; }
        public string version { get {
            return "1.0";
        }}
        public string objectId { get; set; }
        public object data { get; set; }
    }
}