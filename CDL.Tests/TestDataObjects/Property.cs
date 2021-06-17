using System;
using System.Text.Json;

namespace CDL.Tests.TestDataObjects
{
    public class Property
    {
        public int Id { get; set; }
        public string City { get; set; }
        public string Street { get; set; }
        public int BuildingNumber { get; set; }
        public Nullable<int> FlatNumber { get; set; }
        public string Type { get; set; }
        public decimal Adres { get; set; }

        public string ToJSONString() 
        {
            return JsonSerializer.Serialize<Property>(this);
        }
    }
}
