mod common;
use ankurah_core::storage::SledStorageEngine;
use ankurah_core::{model::ScopedRecord, node::Node};
use anyhow::Result;

use common::{Album, AlbumRecord};
use std::sync::Arc;
#[tokio::test]
async fn basic_where_clause() -> Result<()> {
    let client = Arc::new(Node::new(Box::new(SledStorageEngine::new_test().unwrap())));

    let id = {
        let trx = client.begin();
        let id = trx
            .create(&Album {
                name: "Walking on a Dream".into(),
                year: "2008".into(),
            })
            .await
            .id();
        trx.create(&Album {
            name: "Ice on the Dune".into(),
            year: "2013".into(),
        })
        .await;
        trx.create(&Album {
            name: "Two Vines".into(),
            year: "2016".into(),
        })
        .await;
        trx.create(&Album {
            name: "Ask That God".into(),
            year: "2024".into(),
        })
        .await;
        trx.commit().await?;
        id
    };

    let albums: ankurah_core::resultset::ResultSet<AlbumRecord> =
        client.fetch("name = 'Walking on a Dream'").await?;

    assert_eq!(
        albums
            .records
            .iter()
            .map(|active_record| active_record.name())
            .collect::<Vec<String>>(),
        vec!["Walking on a Dream".to_string()]
    );

    Ok(())
}

#[cfg(feature = "postgres")]
mod pg_common;

#[cfg(feature = "postgres")]
#[tokio::test]
async fn pg_basic_where_clause() -> Result<()> {
    let (_container, storage_engine) = pg_common::create_postgres_container().await?;
    let client = Arc::new(Node::new(Box::new(storage_engine)));

    {
        let trx = client.begin();

        trx.create(&Album {
            name: "Walking on a Dream".into(),
            year: "2008".into(),
        })
        .await;
        trx.create(&Album {
            name: "Ice on the Dune".into(),
            year: "2013".into(),
        })
        .await;
        trx.create(&Album {
            name: "Two Vines".into(),
            year: "2016".into(),
        })
        .await;
        trx.create(&Album {
            name: "Ask That God".into(),
            year: "2024".into(),
        })
        .await;

        trx.commit().await?;
    };

    // The next step is to make this work:
    let albums: ankurah_core::resultset::ResultSet<AlbumRecord> =
        client.fetch("name = 'Walking on a Dream'").await?;

    assert_eq!(
        albums
            .records
            .iter()
            .map(|active_record| active_record.name())
            .collect::<Vec<String>>(),
        vec!["Walking on a Dream".to_string()]
    );

    Ok(())
}
