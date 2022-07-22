mod config;
mod entities;

use entities::{Entity, DbMetaData, /*DbCubeData,*/ DbState};

use libfj::robocraft_simple::FactoryAPI;

fn main() {
    let config = config::parse();
    if config.verbose {
        println!("Opening & building database, roboshield be damned");
    }
    let mut db = rusqlite::Connection::open(
        &config.database.unwrap_or("rc_archive.db".to_owned())
    ).unwrap();
    // build database structure
    db.execute_batch(
    "BEGIN;
    CREATE TABLE IF NOT EXISTS ROBOT_METADATA (
        id INTEGER NOT NULL PRIMARY KEY,
        name TEXT NOT NULL,
        description TEXT NOT NULL,
        thumbnail TEXT NOT NULL,
        added_by TEXT NOT NULL,
        added_by_display_name TEXT NOT NULL,
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
        next_page INTEGER NOT NULL
    );
    COMMIT;"
    ).unwrap();
    let state_rows: Vec<rusqlite::Result<DbState>> = db
        .prepare("SELECT * from STATE").unwrap()
        .query_map([], DbState::map_row).unwrap().collect();
    let mut state = if let Some(Ok(state)) = state_rows.get(0) {
        state.to_owned()
    } else {
        DbState {
            id: 0,
            next_page: 0,
        }
    };
    db.execute(
        "INSERT OR REPLACE INTO STATE (
            id, next_page
        ) VALUES (?, ?);",
        state.to_params().as_slice()
    ).unwrap();
    
    // begin scraping
    if config.verbose {
        if state.next_page == 0 {
            println!("Beginning archival process, looking out for T-sticks");
        } else {
            println!("Resuming archival process at page {}, blaming Josh", state.next_page);
        }
    }
    let api = FactoryAPI::new();
    let mut req_builder = api.list_builder()
        .page(state.next_page)
        .no_minimum_cpu()
        .no_maximum_cpu()
        .items_per_page(config.size.unwrap_or(31));
    loop {
        if config.verbose {
            print!("Retrieving page {}", state.next_page);
        }
        let response = req_builder.clone().send().unwrap();
        if response.status_code != 200 {
            eprintln!("Got response status {} from CRF, aborting...", response.status_code);
            break;
        }
        if config.verbose {
            println!("... Got {} robots (beep boop)", response.response.roboshop_items.len());
        }
        let transaction = db.transaction().unwrap();
        {
            let mut metadata_insert = transaction.prepare(
            "INSERT OR REPLACE INTO ROBOT_METADATA (
                id, name, description, thumbnail, added_by, added_by_display_name, expiry_date, cpu, total_robot_ranking, rent_count, buy_count, buyable, featured, combat_rating, cosmetic_rating
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);"
            ).unwrap();
            for robot in response.response.roboshop_items {
                let db_robot: DbMetaData = robot.into();
                metadata_insert.execute(db_robot.to_params().as_slice()).unwrap();
            }
        }
        transaction.commit().unwrap();
        // prepare for next loop iteration
        state.next_page += 1;
        db.execute(
            "INSERT OR REPLACE INTO STATE (
                id, next_page
            ) VALUES (?, ?);",
            state.to_params().as_slice()
        ).unwrap();
        req_builder = req_builder.page(state.next_page);
    }
}
