using System;
using System.Text.Json;

namespace CDL.Tests.TestDataObjects
{
    public class Person
    {
        public int Id { get; set; }
        public string FirstName { get; set; }
        public string LastName { get; set; }
        public double CardId { get; set; }
        public string Email { get; set; }
        public Gander Gander { get; set; }
        public DateTime Birthday { get; set; }
        public string ToJSONString() 
        {
            return JsonSerializer.Serialize<Person>(this);
        }
    }

    public enum Gander
    {
        Male,
        Female
    }
}
