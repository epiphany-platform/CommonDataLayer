using AutoFixture;
using System.Collections.Generic;
using System.Text.Json;

namespace CDL.Tests.TestDataObjects
{
    public class GeneralObject
    {
        public string Id { get; set; }
        public Person Person { get; set; }
        public IList<object> Properties { get; set; }

        public GeneralObject(int familyNumber = 1, int carNumber = 1, int propertyNumber = 1)
        {
            SetupObject();
            Id = _fixture.Create<string>();
            Person = _fixture.Create<Person>();
            for (int i = 0; i < familyNumber; i++)
            {
                Properties.Add(_fixture.Create<FamilyPerson>());
            }

            for (int i = 0; i < carNumber; i++)
            {
                Properties.Add(_fixture.Create<Car>());
            }

            for (int i = 0; i < propertyNumber; i++)
            {
                Properties.Add(_fixture.Create<Property>());
            }
        }

        public string ToJSONString() 
        {
            return JsonSerializer.Serialize<GeneralObject>(this);
        }

        private Fixture _fixture;
        private void SetupObject()
        {
            _fixture = new Fixture();
            Properties = new List<object>();
        }

    }
}
