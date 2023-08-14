use std::fs::OpenOptions;

use crate::entity::peer;
use crate::migration::{Migrator, MigratorTrait};
use crate::utils::error::Error;
use crate::utils::general::get_db_path;
use sea_orm::{
    ColumnTrait, Database as SeaOrmDatabase, DatabaseConnection, EntityTrait,
    NotSet, QueryFilter, Set,
};

async fn get_db_pool() -> Result<DatabaseConnection, Error> {
    let db_path = get_db_path();
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&db_path)?;
    let db_str = format!("sqlite:{}", &db_path);
    let pool = SeaOrmDatabase::connect(&db_str).await?;
    Ok(pool)
}

#[derive(Clone)]
pub struct Database {
    pool: DatabaseConnection,
}

impl Database {
    pub async fn new() -> Result<Self, Error> {
        get_db_pool().await.map(|pool| Self { pool })
    }

    pub async fn apply_migrations(&self) -> Result<(), Error> {
        Ok(Migrator::up(&self.pool, None).await?)
    }

    pub async fn insert_peer(
        &mut self,
        pub_key: &String,
        hostname: &String,
        ip: &String,
    ) -> Result<(), Error> {
        let peer = peer::Entity::find()
            .filter(peer::Column::PubKey.contains(pub_key))
            .one(&self.pool)
            .await?;
        if let None = peer {
            let peer = peer::ActiveModel {
                id: NotSet,
                pub_key: Set(pub_key.to_owned()),
                hostname: Set(hostname.to_owned()),
                ip: Set(ip.to_owned()),
            };
            peer::Entity::insert(peer).exec(&self.pool).await?;
        }
        Ok(())
    }
    pub async fn get_peers_ip(&self) -> Result<Vec<String>, Error> {
        let peers: Vec<String> = peer::Entity::find()
            .all(&self.pool)
            .await?
            .into_iter()
            .map(|peer| peer.ip)
            .collect();
        Ok(peers)
    }
}
