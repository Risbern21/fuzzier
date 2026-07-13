extern crate walkdir;

use std::{collections::HashMap, io, path::PathBuf, sync::Arc};

use embed_anything::{embed_query, embeddings::embed::Embedder};
use serde_json::Value;
use uuid::Uuid;
use vecstore::{Metadata, Neighbor, Query, VecStore};

#[derive(Debug, Clone)]
pub enum Error {
    OprationCancelled,
    VectorStoreError(String),
    IOError(io::ErrorKind),
}

pub async fn find_similar_images(query: String, limit: usize) -> Result<Vec<Neighbor>, Error> {
    let store = VecStore::open("fuzzier-store.db").unwrap();

    let query_embedding = get_query_embedding(&query).await.unwrap();

    let query = Query::new(query_embedding).with_limit(limit);

    let results = match store.query(query) {
        Ok(results) => results,
        Err(_) => {
            vec![]
        }
    };

    Ok(results)
}

async fn get_query_embedding(query: &str) -> Result<Vec<f32>, anyhow::Error> {
    let text_embedder =
        Embedder::from_pretrained_hf("openai/clip-vit-base-patch16", None, None, None)?;

    let embed_data = embed_query(&[query], &text_embedder, None).await.unwrap();

    let embedding = embed_data
        .into_iter()
        .next()
        .and_then(|e| e.embedding.to_dense().ok())
        .ok_or_else(|| anyhow::anyhow!("no embedding returned for the query"))?;

    Ok(embedding)
}

pub async fn embed_image_directory(directory: PathBuf) {
    let embedder = Arc::new(
        Embedder::from_pretrained_hf("openai/clip-vit-base-patch16", None, None, None).unwrap(),
    );
    let embeddings = embed_anything::embed_image_directory(directory, &embedder, None, None)
        .await
        .unwrap()
        .expect("no embeddings created");

    for embedding in embeddings {
        store_image_embedding(
            embedding.embedding.to_dense().unwrap(),
            embedding.metadata.expect("no metadata provuided"),
        )
        .unwrap();
    }
}

fn store_image_embedding(
    embedding: Vec<f32>,
    image_metadata: HashMap<String, String>,
) -> Result<(), anyhow::Error> {
    let mut store = VecStore::open("fuzzier-store.db")?;

    let fields = image_metadata
        .into_iter()
        .map(|(k, v)| (k, Value::String(v)))
        .collect();

    let metadata = Metadata { fields };

    match store.upsert(Uuid::new_v4().to_string(), embedding, metadata) {
        Ok(_) => println!("inserted successfully"),
        Err(err) => println!("error while inserting {:?}", err),
    };

    store.save().unwrap();

    Ok(())
}
