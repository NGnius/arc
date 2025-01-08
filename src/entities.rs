use libfj::robocraft::{FactoryRobotListInfo, FactoryRobotGetInfo};
use std::convert::From;

pub fn build_database(db: &mut rusqlite::Connection) -> rusqlite::Result<()> {
    db.execute_batch(
    "BEGIN;
    CREATE TABLE IF NOT EXISTS ROBOT_METADATA (
        id INTEGER NOT NULL PRIMARY KEY,
        name TEXT NOT NULL,
        description TEXT NOT NULL,
        thumbnail TEXT NOT NULL,
        added_by TEXT NOT NULL,
        added_by_display_name TEXT NOT NULL,
        added_date TEXT NOT NULL,
        expiry_date TEXT NOT NULL,
        cpu INTEGER NOT NULL,
        total_robot_ranking INTEGER NOT NULL,
        rent_count INTEGER NOT NULL,
        buy_count INTEGER NOT NULL,
        buyable INTEGER NOT NULL,
        featured INTEGER NOT NULL,
        combat_rating REAL NOT NULL,
        cosmetic_rating REAL NOT NULL
    );
    CREATE TABLE IF NOT EXISTS ROBOT_CUBES (
        id INTEGER NOT NULL PRIMARY KEY,
        cube_data TEXT NOT NULL,
        colour_data TEXT NOT NULL,
        cube_amounts TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS STATE (
        id INTEGER NOT NULL PRIMARY KEY,
        next_page INTEGER NOT NULL,
        last_page_size INTEGER NOT NULL,
        last_sequential_id INTEGER NOT NULL
    );
    COMMIT;"
    )
}

pub trait Entity: Sized {
    fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Self>;

    fn to_params(&self) -> Vec<&dyn rusqlite::ToSql>;

    #[allow(dead_code)]
    fn id(&self) -> usize;
}

#[derive(Clone, Debug)]
pub struct DbMetaData {
    pub id: usize,
    pub name: String,
    pub description: String,
    pub thumbnail: String,
    pub added_by: String,
    pub added_by_display_name: String,
    pub added_date: String,
    pub expiry_date: String,
    pub cpu: usize,
    pub total_robot_ranking: usize,
    pub rent_count: usize,
    pub buy_count: usize,
    pub buyable: bool,
    pub featured: bool,
    pub combat_rating: f32,
    pub cosmetic_rating: f32,
}

impl Entity for DbMetaData {
    /*
    id INTEGER NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    thumbnail TEXT NOT NULL,
    added_by TEXT NOT NULL,
    added_by_display_name TEXT NOT NULL,
    added_date TEXT NOT NULL,
    expiry_date TEXT NOT NULL,
    cpu INTEGER NOT NULL,
    total_robot_ranking INTEGER NOT NULL,
    rent_count INTEGER NOT NULL,
    buy_count INTEGER NOT NULL,
    buyable INTEGER NOT NULL,
    featured INTEGER NOT NULL,
    combat_rating REAL NOT NULL,
    cosmetic_rating REAL NOT NULL,
    */
    fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            thumbnail: row.get(3)?,
            added_by: row.get(4)?,
            added_by_display_name: row.get(5)?,
            added_date: row.get(6)?,
            expiry_date: row.get(7)?,
            cpu: row.get(8)?,
            total_robot_ranking: row.get(9)?,
            rent_count: row.get(10)?,
            buy_count: row.get(11)?,
            buyable: row.get(12)?,
            featured: row.get(13)?,
            combat_rating: row.get(14)?,
            cosmetic_rating: row.get(15)?,
        })
    }

    fn to_params(&self) -> Vec<&dyn rusqlite::ToSql> {
        vec![
            &self.id,
            &self.name,
            &self.description,
            &self.thumbnail,
            &self.added_by,
            &self.added_by_display_name,
            &self.added_date,
            &self.expiry_date,
            &self.cpu,
            &self.total_robot_ranking,
            &self.rent_count,
            &self.buy_count,
            &self.buyable,
            &self.featured,
            &self.combat_rating,
            &self.cosmetic_rating,
        ]
    }

    fn id(&self) -> usize {
        self.id
    }
}

impl From<FactoryRobotListInfo> for DbMetaData {
    fn from(other: FactoryRobotListInfo) -> Self {
        Self {
            id: other.item_id,
            name: other.item_name,
            description: other.item_description,
            thumbnail: other.thumbnail,
            added_by: other.added_by,
            added_by_display_name: other.added_by_display_name,
            added_date: other.added_date,
            expiry_date: other.expiry_date,
            cpu: other.cpu,
            total_robot_ranking: other.total_robot_ranking,
            rent_count: other.rent_count,
            buy_count: other.buy_count,
            buyable: other.buyable,
            featured: other.featured,
            combat_rating: other.combat_rating,
            cosmetic_rating: other.cosmetic_rating,
        }
    }
}

impl From<FactoryRobotGetInfo> for DbMetaData {
    fn from(other: FactoryRobotGetInfo) -> Self {
        Self {
            id: other.item_id,
            name: other.item_name,
            description: other.item_description,
            thumbnail: other.thumbnail,
            added_by: other.added_by,
            added_by_display_name: other.added_by_display_name,
            added_date: other.added_date,
            expiry_date: other.expiry_date,
            cpu: other.cpu,
            total_robot_ranking: other.total_robot_ranking,
            rent_count: other.rent_count,
            buy_count: other.buy_count,
            buyable: other.buyable,
            featured: other.featured,
            combat_rating: other.combat_rating,
            cosmetic_rating: other.cosmetic_rating,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DbCubeData {
    pub id: usize,
    pub cube_data: String,
    pub colour_data: String,
    pub cube_amounts: String,
}

impl Entity for DbCubeData {
    /*
    id INTEGER NOT NULL PRIMARY KEY,
    cube_data TEXT NOT NULL,
    colour_data TEXT NOT NULL,
    cube_amounts TEXT NOT NULL,
    */
    fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            cube_data: row.get(1)?,
            colour_data: row.get(2)?,
            cube_amounts: row.get(3)?,
        })
    }

    fn to_params(&self) -> Vec<&dyn rusqlite::ToSql> {
        vec![
            &self.id,
            &self.cube_data,
            &self.colour_data,
            &self.cube_amounts,
        ]
    }

    fn id(&self) -> usize {
        self.id
    }
}

impl From<FactoryRobotGetInfo> for DbCubeData {
    fn from(other: FactoryRobotGetInfo) -> Self {
        Self {
            id: other.item_id,
            cube_data: other.cube_data,
            colour_data: other.colour_data,
            cube_amounts: other.cube_amounts,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DbState {
    pub id: usize,
    pub next_page: isize,
    pub last_page_size: isize,
    pub last_sequential_id: usize,
}


impl Entity for DbState {
    /*
    id INTEGER NOT NULL PRIMARY KEY,
    next_page INTEGER NOT NULL,
    last_page_size INTEGER NOT NULL,
    last_sequential_id INTEGER NOT NULL,
    */
    fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            next_page: row.get(1)?,
            last_page_size: row.get(2)?,
            last_sequential_id: row.get(3)?,
        })
    }

    fn to_params(&self) -> Vec<&dyn rusqlite::ToSql> {
        vec![
            &self.id,
            &self.next_page,
            &self.last_page_size,
            &self.last_sequential_id,
        ]
    }

    fn id(&self) -> usize {
        self.id
    }
}
