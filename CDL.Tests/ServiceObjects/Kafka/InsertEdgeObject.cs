using System.Collections.Generic;

namespace CDL.Tests.MessageBroker.Kafka
{
    public class InsertEdgeObject
    {
        public string relation_id { get; set; }
        public string parent_object_id { get; set; }
        public IList<string> child_object_ids { get; set; }
    }
}