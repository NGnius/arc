mod config;
mod entities;

use config::CliArgs;
use entities::{Entity, DbMetaData, DbCubeData, DbState};

use rusqlite::Connection;

use libfj::robocraft_simple::FactoryAPI;
use libfj::robocraft::{FactoryRobotGetInfo, FactoryInfo};

const DEFAULT_PAGE_SIZE: isize = 100;
const PERIOD: usize = 100;

fn main() {
    let config = config::parse();
    if config.verbose {
        println!("Opening & building database, roboshield be damned");
    }
    let mut db = rusqlite::Connection::open(
        &config.database.clone().unwrap_or("rc_archive.db".to_owned())
    ).unwrap();
    // build database structure
    entities::build_database(&mut db).unwrap();
    let mut state = build_state(&mut db, &config);

    save_state(&mut db, &state);
    
    // begin scraping
    if config.verbose {
        if state.next_page == 0 {
            println!("Beginning archival process, looking out for T-sticks");
        } else {
            println!("Resuming archival process at page {}, blaming Josh", state.next_page);
        }
    }
    let api = FactoryAPI::new();
    search_bots(&mut db, &config, &mut state, &api);
    if config.known {
        if config.verbose {
            println!("Downloading robot cubes data for all known robots");
        }
        download_missing_bots(&mut db, &config, &api);
    } else {
        if config.verbose {
            println!("Looking for non-searchable bots, activating windowmaker module");
        }
        download_all_bots(&mut db, &mut state, &config, &api);
    }
    if config.verbose {
        println!("Done.");
    }
}

fn build_state(db: &mut Connection, config: &CliArgs) -> DbState {
    if config.new || config.known {
        DbState {
            id: 0,
            next_page: 0,
            last_page_size: config.size.unwrap_or(DEFAULT_PAGE_SIZE),
            last_sequential_id: usize::MAX,
        }
    } else {
        let state_rows: Vec<rusqlite::Result<DbState>> = db
            .prepare("SELECT * from STATE").unwrap()
            .query_map([], DbState::map_row).unwrap().collect();
        if let Some(Ok(state)) = state_rows.get(0) {
            if let Some(page_size) = config.size {
                if state.last_page_size != page_size {
                    DbState {
                        id: 0,
                        next_page: 0,
                        last_page_size: page_size,
                        last_sequential_id: usize::MAX,
                    }
                } else {
                    state.to_owned()
                }
            } else {
                state.to_owned()
            }
        } else {
            DbState {
                id: 0,
                next_page: 0,
                last_page_size: config.size.unwrap_or(DEFAULT_PAGE_SIZE),
                last_sequential_id: usize::MAX,
            }
        }
    }
}

fn save_state(db: &mut Connection, state: &DbState) {
    db.execute(
        "INSERT OR REPLACE INTO STATE (
            id, next_page, last_page_size, last_sequential_id
        ) VALUES (?, ?, ?, ?);",
        state.to_params().as_slice()
    ).unwrap();
}

fn search_bots(db: &mut Connection, config: &CliArgs, state: &mut DbState, api: &FactoryAPI) {
    let mut req_builder = api.list_builder()
        .page(state.next_page)
        .no_minimum_cpu()
        .no_maximum_cpu()
        .order(libfj::robocraft::FactoryOrderType::Added)
        .movement_raw("100000,200000,300000,400000,500000,600000,700000,800000,900000,1000000,1100000,1200000".to_owned())
        .weapon_raw("10000000,20000000,25000000,30000000,40000000,50000000,60000000,65000000,70100000,75000000".to_owned())
        .default_page(false)
        .items_per_page(state.last_page_size);
    loop {
        if config.verbose {
            print!("Retrieving page {}", state.next_page);
        }
        let response = req_builder.clone().send().unwrap();
        if response.status_code != 200 {
            eprintln!("Got response status {}, self-destructing...", response.status_code);
            break;
        }
        if response.response.roboshop_items.is_empty() {
            if config.verbose {
                println!("... Got response page with no items, search has been defeated!");
            }
            break;
        }
        if config.verbose {
            println!("... Got {} robots (beep boop)", response.response.roboshop_items.len());
        }
        let transaction = db.transaction().unwrap();
        {
            let mut metadata_insert = transaction.prepare(
            "INSERT OR REPLACE INTO ROBOT_METADATA (
                id, name, description, thumbnail, added_by, added_by_display_name, added_date, expiry_date, cpu, total_robot_ranking, rent_count, buy_count, buyable, featured, combat_rating, cosmetic_rating
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);"
            ).unwrap();
            for robot in response.response.roboshop_items {
                let db_robot: DbMetaData = robot.into();
                metadata_insert.execute(db_robot.to_params().as_slice()).unwrap();
            }
        }
        transaction.commit().unwrap();
        // prepare for next loop iteration
        state.next_page += 1;
        save_state(db, state);
        if config.new {
            if config.verbose {
                println!("Stopping search before older robots are found");
            }
            break;
        }
        req_builder = req_builder.page(state.next_page);
    }
}

fn download_missing_bots(db: &mut Connection, config: &CliArgs, api: &FactoryAPI) {
    let missing_bots: Vec<rusqlite::Result<DbMetaData>> = db
        .prepare(
            "SELECT * FROM ROBOT_METADATA rm WHERE rm.id NOT IN (SELECT id from ROBOT_CUBES rc);"
        ).unwrap()
        .query_map([], DbMetaData::map_row).unwrap().collect();
    if config.verbose {
        println!("Found {} robots which need their cubes downloaded", missing_bots.len());
    }
    for bot in missing_bots {
        if let Ok(bot) = bot {
            let response = api.get(bot.id).unwrap();
            persist_bot(db, config, response);
        }
    }
}

fn persist_bot(db: &mut Connection, config: &CliArgs, response: FactoryInfo<FactoryRobotGetInfo>) -> bool {
    if response.status_code != 200 {
        eprintln!("Got response status {}, self-destructing...", response.status_code);
        return false;
    }
    let robo_data = response.response;
    if config.new {
        if config.verbose {
            println!("Found new robot #{} (`{}` by {}, {} CPU)", robo_data.item_id, robo_data.item_name, robo_data.added_by_display_name, robo_data.cpu);
        } else {
            println!("Found new robot #{}", robo_data.item_id);
        }
    }
    let robot_meta: DbMetaData = robo_data.clone().into();
    db.execute(
        "INSERT OR REPLACE INTO ROBOT_METADATA (
        id, name, description, thumbnail, added_by, added_by_display_name, added_date, expiry_date, cpu, total_robot_ranking, rent_count, buy_count, buyable, featured, combat_rating, cosmetic_rating
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);",
        robot_meta.to_params().as_slice()
    ).unwrap();
    let robot_cubes: DbCubeData = robo_data.into();
    db.execute(
        "INSERT OR REPLACE INTO ROBOT_CUBES (
        id, cube_data, colour_data, cube_amounts
        ) VALUES (?, ?, ?, ?);",
        robot_cubes.to_params().as_slice()
    ).unwrap();
    true
}

fn download_all_bots(db: &mut Connection, state: &mut DbState, config: &CliArgs, api: &FactoryAPI) {
    let latest_bot_row: Vec<rusqlite::Result<DbMetaData>> = db
        .prepare("SELECT * from ROBOT_METADATA rm ORDER BY rm.id DESC LIMIT 1;").unwrap()
        .query_map([], DbMetaData::map_row).unwrap().collect();

    let latest_cube_row: Vec<rusqlite::Result<DbCubeData>> = db
        .prepare("SELECT * from ROBOT_CUBES rc ORDER BY rc.id DESC LIMIT 1;").unwrap()
        .query_map([], DbCubeData::map_row).unwrap().collect();

    let oldest_cube_row: Vec<rusqlite::Result<DbCubeData>> = db
        .prepare("SELECT * from ROBOT_CUBES rc ORDER BY rc.id ASC LIMIT 1;").unwrap()
        .query_map([], DbCubeData::map_row).unwrap().collect();

    if let Some(Ok(highest_bot)) = latest_bot_row.get(0) {
        if let Some(Ok(highest_cubes)) = latest_cube_row.get(0) {
            if let Some(Ok(lowest_cubes)) = oldest_cube_row.get(0) {
                let highest_id = highest_bot.id;
                let highest_cube_id = highest_cubes.id;
                let lowest_cube_id = lowest_cubes.id;
                // NOTE: IDs are gone through sequentially instead of just retrieving the known ones
                // because the default user cannot search for non-buyable robots, despite them existing.
                // This creates gaps in known (i.e. searchable) IDs, despite IDs being sequential.
                if config.new {
                    for id in highest_cube_id+1..=highest_id {
                        if let Ok(response) = api.get(id) {
                            if !persist_bot(db, config, response) {
                                break;
                            }
                        }
                    }
                } else {
                    if config.verbose {
                        println!("Most recent bot has id #{}, existing data for #{} (ignoring down to #{}) to #{}", highest_id, state.last_sequential_id, lowest_cube_id, highest_cube_id);
                    }
                    for id in (0..=highest_id).rev() {
                        if id <= highest_cube_id && id >= state.last_sequential_id {
                            continue;
                        }
                        if let Ok(response) = api.get(id) {
                            if !persist_bot(db, config, response) {
                                break;
                            }
                            if state.last_sequential_id - id >= PERIOD {
                                state.last_sequential_id = id - (id % PERIOD);
                                save_state(db, state);
                            }
                        }
                        if config.verbose && id % PERIOD == 0 {
                            println!("Done bot #{}, last persistent id #{}", id, state.last_sequential_id);
                        }
                    }
                }
            }
        }
    } else {
        eprintln!("No robots in database, cannot brute-force IDs!");
    }
}
