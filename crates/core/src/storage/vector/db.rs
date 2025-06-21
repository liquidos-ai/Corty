#![allow(dead_code)]

use arrow_array::{RecordBatch, RecordBatchIterator, StringArray};
use arrow_schema::{DataType, Field, Schema};
use color_eyre::Result;
use futures::StreamExt;
use lancedb::{
    embeddings::{
        sentence_transformers::SentenceTransformersEmbeddings, EmbeddingDefinition,
        EmbeddingFunction,
    },
    query::{ExecutableQuery, QueryBase},
    Table,
};
use std::{collections::HashMap, iter::once, sync::Arc};

pub struct VectorDB {
    path: String,
    db: lancedb::Connection,
    embedding: Arc<SentenceTransformersEmbeddings>,
}

impl VectorDB {
    pub async fn connect(path: &str) -> Result<Self> {
        let db = lancedb::connect(path).execute().await?;
        let embedding = Arc::new(SentenceTransformersEmbeddings::builder().build()?);
        db.embedding_registry()
            .register("sentence-transformers", embedding.clone())?;
        println!("Done new!");
        Ok(Self {
            path: path.to_string(),
            db,
            embedding: embedding.clone(),
        })
    }

    pub async fn create_empty_table(&self, name: &str) -> Result<Table> {
        // Create table with minimal data to establish embedding schema
        let schema = Arc::new(Schema::new(vec![Field::new("item", DataType::Utf8, true)]));
        let placeholder_values = StringArray::from_iter_values(vec!["__temp__".to_string()]);
        let rb = RecordBatch::try_new(schema.clone(), vec![Arc::new(placeholder_values)]).unwrap();
        let rb_iter = Box::new(RecordBatchIterator::new(vec![Ok(rb)], schema));

        let table = self
            .db
            .create_table(name, rb_iter)
            .add_embedding(EmbeddingDefinition::new(
                "item",
                "sentence-transformers",
                Some("embeddings"),
            ))?
            .execute()
            .await?;

        // Clear the placeholder data
        table.delete("item = '__temp__'").await?;
        Ok(table)
    }

    pub async fn create_table_with_data(
        &self,
        name: &str,
        data: HashMap<String, String>,
    ) -> Result<Table> {
        let schema = Arc::new(Schema::new(vec![Field::new("item", DataType::Utf8, true)]));
        let values = StringArray::from_iter_values(
            data.iter()
                .map(|f| f.1.to_string())
                .collect::<Vec<String>>(),
        );
        let rb = RecordBatch::try_new(schema.clone(), vec![Arc::new(values)]).unwrap();
        let rb_iter = Box::new(RecordBatchIterator::new(vec![Ok(rb)], schema));

        let table = self
            .db
            .create_table(name, rb_iter)
            .add_embedding(EmbeddingDefinition::new(
                "item",
                "sentence-transformers",
                Some("embeddings"),
            ))?
            .execute()
            .await?;
        Ok(table)
    }

    pub async fn insert(&self, table: &Table, data: HashMap<String, String>) -> Result<()> {
        let schema = Arc::new(Schema::new(vec![Field::new("item", DataType::Utf8, true)]));
        let values = StringArray::from_iter_values(
            data.iter()
                .map(|f| f.1.to_string())
                .collect::<Vec<String>>(),
        );
        let rb = RecordBatch::try_new(schema.clone(), vec![Arc::new(values)]).unwrap();
        let rb_iter = Box::new(RecordBatchIterator::new(vec![Ok(rb)], schema));
        println!("Adding data");
        table.add(rb_iter).execute().await?;
        Ok(())
    }

    pub async fn get_table(&self, name: &str) -> Table {
        self.db.open_table(name).execute().await.unwrap()
    }

    pub async fn query(&self, query: &str, table: &Table) -> Result<()> {
        let query = Arc::new(StringArray::from_iter_values(once(query)));
        let query_vector = self.embedding.compute_query_embeddings(query)?;
        let mut results = table
            .vector_search(query_vector)?
            .limit(1)
            .execute()
            .await?;

        let rb = results.next().await.unwrap()?;
        let out = rb
            .column_by_name("item")
            .unwrap()
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        let text = out.iter().next().unwrap().unwrap();
        println!("Answer: {}", text);
        Ok(())
    }
}
