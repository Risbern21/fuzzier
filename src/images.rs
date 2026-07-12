extern crate walkdir;

use std::{collections::HashMap, io, path::PathBuf, sync::Arc};

use embed_anything::embeddings::embed::Embedder;
use serde_json::Value;
use uuid::Uuid;
use vecstore::{Metadata, VecStore};
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Clone)]
pub enum Error {
    OprationCancelled,
    VectorStoreError(String),
    IOError(io::ErrorKind),
}

pub async fn find_similar_images(query: String) -> Result<Vec<DirEntry>, Error> {
    let mut files = vec![];

    for file in WalkDir::new("/home/risbern21/Pictures")
        .into_iter()
        .filter_map(|file| file.ok())
    {
        files.push(file);
    }

    Ok(files)
}

fn store_image_embedding(
    embedding: Vec<f32>,
    image_metadata: HashMap<String, String>,
) -> Result<(), anyhow::Error> {
    println!("image metadata is : {:?}", image_metadata);
    println!();
    println!("embeddings : {:?}", embedding);

    let mut store = VecStore::open("fuzzier-store.db")?;

    let fields = image_metadata
        .into_iter()
        .map(|(k, v)| (k, Value::String(v)))
        .collect();

    let metadata = Metadata { fields };

    let _ = store.upsert(Uuid::new_v4().to_string(), embedding, metadata);

    Ok(())
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
