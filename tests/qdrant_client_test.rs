#[cfg(test)]
mod tests {
    use qdrant_client::qdrant::{
        Condition, CreateCollectionBuilder, Distance, Filter, PointStruct, SearchPointsBuilder,
        UpsertPointsBuilder, VectorParamsBuilder,
    };
    use qdrant_client::{Payload, Qdrant};
    use serde_json::json;

    #[tokio::test] // Use tokio's test macro for async support
    async fn test_qdrant_operations() -> Result<(), Box<dyn std::error::Error>> {
        // Initialize Qdrant client
        let client = Qdrant::from_url("http://localhost:6334").build()?;

        // Clean up any existing collection
        let collection_name = "test";
        let _ = client.delete_collection(collection_name).await;

        // Create a new collection
        client
            .create_collection(
                CreateCollectionBuilder::new(collection_name)
                    .vectors_config(VectorParamsBuilder::new(10, Distance::Cosine)),
            )
            .await?;

        // Insert a point
        let payload: Payload = json!({
            "foo": "Bar",
            "bar": 12,
            "baz": {
                "qux": "quux"
            }
        })
        .try_into()
        .unwrap();
        let points = vec![PointStruct::new(0, vec![12.; 10], payload)];
        client
            .upsert_points(UpsertPointsBuilder::new(collection_name, points))
            .await?;

        // Search for the inserted point
        let search_result = client
            .search_points(
                SearchPointsBuilder::new(collection_name, [11.; 10], 10)
                    .filter(Filter::all([Condition::matches("bar", 12)]))
                    .with_payload(true),
            )
            .await?;

        // Verify the search result
        assert!(!search_result.result.is_empty());
        let found_point = search_result.result.into_iter().next().unwrap();
        let payload = found_point.payload;
        assert_eq!(payload.get("foo").unwrap().as_str(), Some("Bar"));
        assert_eq!(payload.get("bar").unwrap().as_i64(), Some(12));

        // Clean up
        client.delete_collection(collection_name).await?;

        Ok(())
    }
}
