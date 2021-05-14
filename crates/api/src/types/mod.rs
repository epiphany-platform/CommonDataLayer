pub mod data;
pub mod report;
pub mod schema;
pub mod view;

pub trait IntoQueried {
    type Queried;
    fn into_queried(self) -> Self::Queried;
}

impl<T> IntoQueried for Option<T>
where
    T: IntoQueried,
{
    type Queried = Option<T::Queried>;

    fn into_queried(self) -> Self::Queried {
        self.map(|s| s.into_queried())
    }
}

impl<T> IntoQueried for Vec<T>
where
    T: IntoQueried,
{
    type Queried = Vec<T::Queried>;

    fn into_queried(self) -> Self::Queried {
        self.into_iter().map(|s| s.into_queried()).collect()
    }
}

impl<T> IntoQueried for async_graphql::Json<T>
where
    T: IntoQueried,
{
    type Queried = T::Queried;

    fn into_queried(self) -> Self::Queried {
        self.0.into_queried()
    }
}
