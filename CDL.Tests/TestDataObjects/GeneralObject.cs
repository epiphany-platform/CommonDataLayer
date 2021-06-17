using AutoFixture;
using System.Collections.Generic;
using System.Text.Json;

namespace CDL.Tests.TestDataObjects
{
    public class GeneralObject
    {
        public int Id { get; set; }
        public Person Person { get; set; }
        public IList<Property> Properties { get; set; }
        
        public string ToJSONString() 
        {
            return JsonSerializer.Serialize<GeneralObject>(this);
        }
    }
}
