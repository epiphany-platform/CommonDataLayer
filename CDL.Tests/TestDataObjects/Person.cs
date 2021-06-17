using System;

namespace CDL.Tests.TestDataObjects
{
    public class Person
    {
        public string FirstName { get; set; }
        public string LastName { get; set; }
        public string Email { get; set; }
        public Gander Gander { get; set; }
        public DateTime Birthday { get; set; }
    }

    public enum Gander
    {
        Male,
        Female
    }
}
