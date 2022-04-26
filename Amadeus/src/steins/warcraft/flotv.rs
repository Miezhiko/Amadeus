use crate::types::team::DiscordPlayer;

mod schema {
  cynic::use_schema!("../schemas/flotv.graphql");
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    schema_path = "../schemas/flotv.graphql",
    query_module = "schema"
)]
pub struct Player {
  pub name: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    schema_path = "../schemas/flotv.graphql",
    query_module = "schema"
)]
pub struct GameSnapshot {
  pub id: i32,
  pub map_name: String,
  pub players: Vec<Player>
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    schema_path = "../schemas/flotv.graphql",
    graphql_type = "QueryRoot"
)]
struct GameSnapshotQuery {
  games: Vec<GameSnapshot>,
}


#[cynic::schema_for_derives(file = "../schemas/flotv.graphql", module = "schema")]
mod queries {
  use super::schema;

  #[derive(cynic::FragmentArguments, Debug)]
  pub struct CreateObserverTokenArguments {
    pub game_id: i32,
  }

  #[derive(cynic::QueryFragment, Debug)]
  pub struct ObserverTokenPayload {
    pub token: String,
  }

  #[derive(cynic::QueryFragment, Debug)]
  #[cynic(
      graphql_type = "MutationRoot",
      argument_struct = "CreateObserverTokenArguments"
  )]
  pub struct CreateObserverToken {
    #[arguments(game_id = args.game_id)]
    pub create_observer_token: ObserverTokenPayload,
  }
}

fn build_query() -> cynic::Operation<'static, GameSnapshotQuery> {
  use cynic::QueryBuilder;
  GameSnapshotQuery::build(())
}

fn build_query_mutation(game_id: i32) -> cynic::Operation<'static, queries::CreateObserverToken> {
  use cynic::MutationBuilder;
  use queries::{CreateObserverToken, CreateObserverTokenArguments};

  CreateObserverToken::build(&CreateObserverTokenArguments {
    game_id
  })
}

async fn run_query(rqcl: &reqwest::Client)
    -> anyhow::Result< cynic::GraphQlResponse<GameSnapshotQuery> > {
  use cynic::http::ReqwestExt;
  let query = build_query();
  rqcl.post("https://stats.w3flo.com")
      .run_graphql(query)
      .await
      .map_err(|e| anyhow!("Failed to get flotv response {e}"))
}

async fn run_query_mutation(rqcl: &reqwest::Client, game_id: i32)
    -> anyhow::Result< cynic::GraphQlResponse<queries::CreateObserverToken> > {
  use cynic::http::ReqwestExt;
  let query = build_query_mutation(game_id);
  rqcl.post("https://stats.w3flo.com")
      .run_graphql(query)
      .await
      .map_err(|e| anyhow!("Failed to get flotv response {e}"))
}

pub async fn get_flotv( rqcl: &reqwest::Client
                      , playing: &[&DiscordPlayer]
                      ) -> anyhow::Result< Option<String> > {
  let grapql_result = run_query(rqcl).await?;
  if let Some(data) = grapql_result.data {
    for game in data.games {
      if playing.iter().any(|p|
        game.players.iter().any(|pp|
          p.player.battletag == pp.name
        )
      ) {
        let grapql_mutation_result = run_query_mutation(rqcl, game.id).await?;
        if let Some(r) = grapql_mutation_result.data {
          return Ok( Some( r.create_observer_token.token ) );
        } else {
          return Err( anyhow!("Failed to generate observer token") );
        }
      }
    }
  }
  Ok( None )
}
