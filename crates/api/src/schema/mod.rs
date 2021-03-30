pub mod context;
pub mod mutation;
pub mod query;
pub mod subscription;
pub mod utils;

use juniper::RootNode;
use mutation::Mutation;
use query::Query;
use subscription::Subscription;

pub type GQLSchema = RootNode<'static, Query, Mutation, Subscription>;

pub fn schema() -> GQLSchema {
    GQLSchema::new(Query, Mutation, Subscription)
}
