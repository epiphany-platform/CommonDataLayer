using System.Text.Json;

namespace CDL.Tests.TestDataObjects
{
    public class Car
    {
        public int Id { get; set; }
        public string Make { get; set; }
        public string Model { get; set; }
        public string Licence { get; set; }
        public int Age { get; set; }
        public decimal Color { get; set; }
        public BodyType BodyType { get; set; }

        public string ToJSONString() 
        {
            return JsonSerializer.Serialize<Car>(this);
        }       
    }

    public enum BodyType
    {
        Coupe,
        Sedan,
        Hatchback,
        Wagon,
        SUV
    }
}
