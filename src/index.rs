use super::*;

lazy_static! {
  static ref SCHEMA: Value = json!({
    "mappings": {
      "properties": {
        "id": { "type": "keyword" },
        "name": { "type": "text" },
        "description": { "type": "text" },
        "license": { "type": "text" },
        "documentation": { "type": "text" },
        "homepage": { "type": "text" },
        "repository": { "type": "text" },
        "downloads": { "type": "long" },
        "recent_downloads": { "type": "long" },
        "categories": { "type": "text" },
        "keywords": { "type": "text" },
        "versions": { "type": "long" },
        "max_version": { "type": "text" },
        "max_stable_version": { "type": "text" },
        "links": {
          "properties": {
            "owner_team": { "type": "text" },
            "owner_user": { "type": "text" },
            "owners": { "type": "text" },
            "reverse_dependencies": { "type": "text" },
            "version_downloads": { "type": "text" },
            "versions": { "type": "text" }
          }
        },
        "created_at": { "type": "date" },
        "updated_at": { "type": "date" },
        "exact_match": { "type": "boolean" }
      }
    }
  });
}

#[derive(Debug)]
pub(crate) struct Index {
  client: Elasticsearch,
}

impl Index {
  const INDEX_ID: &str = "crate-index";

  pub(crate) fn open() -> Result<Self> {
    Ok(Self {
      client: Elasticsearch::new(Transport::single_node(
        "http://localhost:9200",
      )?),
    })
  }

  pub(crate) async fn ingest(&self, item: Crate) -> Result {
    info!("Ingesting crate: {}", item.name);

    if !self.client.has_index(Index::INDEX_ID).await? {
      self
        .client
        .create_index(Index::INDEX_ID, SCHEMA.to_owned())
        .await?
        .raise_for_status(StatusCode::OK)?;
    }

    if self.client.has_document(Index::INDEX_ID, &item.id).await? {
      return Ok(());
    }

    self
      .client
      .index_document(Index::INDEX_ID, &item.id, &item)
      .await?
      .raise_for_status(StatusCode::CREATED)?;

    Ok(())
  }

  pub(crate) async fn search(&self, query: &str) -> Result<serde_json::Value> {
    info!("Received query: {query}");

    let response = self
      .client
      .query(
        Index::INDEX_ID,
        json!({
          "query": {
            "query_string": {
              "query": query
            }
          }
        }),
      )
      .await?
      .raise_for_status(StatusCode::OK)?;

    Ok(response.json().await?)
  }
}
