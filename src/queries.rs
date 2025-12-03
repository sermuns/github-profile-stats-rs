use bincode::{Decode, Encode};
use graphql_client::GraphQLQuery;

#[allow(clippy::upper_case_acronyms)]
type URI = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/schema.docs.graphql",
    query_path = "src/basicstats.graphql",
    response_derives = "Debug"
)]
pub struct BasicStats;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/schema.docs.graphql",
    query_path = "src/repostats.graphql",
    response_derives = "Debug,Encode,Decode"
)]
pub struct RepoStats;
