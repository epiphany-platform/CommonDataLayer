namespace CDL.Tests.MessageBroker.Kafka
{
    public class InsertObject
    {
        public string schemaId { get; set; }
        public string objectId { get; set; }
        public object data { get; set; }
    }
}