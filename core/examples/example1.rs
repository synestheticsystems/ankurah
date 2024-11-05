use std::sync::Arc;

use ankurah_core::model::Record;
use ankurah_core::storage::SledStorageEngine;
use ankurah_core::{model::ID, node::Node};
use ankurah_derive::Model;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{info, Level};

#[derive(Model, Debug, Serialize, Deserialize)] // This line now uses the Model derive macro
pub struct Album {
    name: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    // Gradually uncomment this example as we add functionality
    // let server = Node::new();
    let mut client = Node::new(Box::new(SledStorageEngine::new().unwrap()));
    let client = Arc::new(client);

    // client.local_connect(&server);

    // let client_albums = client.collection::<Album>("album");
    // let server_albums = server.collection::<Album>("album");

    // Lets get signals working after we have the basics
    // tokio::spawn(server_albums.signal().for_each(|changeset| {
    //     println!("Album recordset changed on server: {}", changeset.operation);
    // }));

    // tokio::spawn(client_albums.signal().for_each(|changeset| {
    //     println!("Album recordset changed on client: {}", changeset.operation);
    // }));

    // TODO add macro for this
    // let album = create_album! {
    //     client,
    //     name: "The Dark Sid of the Moon",
    // };
    let album = {
        let trx = client.begin();
        let album = AlbumRecord::new(
            &client,
            Album {
                name: "The Dark Sid of the Moon".to_string(),
            },
        );

        info!("Album created: {:?}", album);
        album.name().insert(12, "e");
        trx.commit().unwrap();

        client
            .bucket("album")
            .set_state(album.id(), album.record_state())?;
        {
            // info!("ababa");
            let trx = client.begin();
            let test = trx.get::<AlbumRecord>("album", album.id())?;
            assert_eq!(test.name().value(), "The Dark Side of the Moon");
            println!("test: {:?}", test.name().value());
            // info!("ababa");
        }

        album
    };

    assert_eq!(album.name.value(), "The Dark Side of the Moon");

    use ankurah_core::property::traits::StateSync;
    let update = album.name().get_pending_update();
    println!("Update length: {}", update.unwrap().len());

    // should immediately have two operations - one for the initial insert, and one for the edit
    // assert_eq!(album.operation_count(), 2);
    // assert_eq!(client_albums.operation_count(), 2);
    // assert_eq!(client_albums.record_count(), 1);

    // Both client and server signals should trigger
    Ok(())
}
