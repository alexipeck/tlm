use crate::generic::FileVersion;
use crate::model::WorkerModel;
use crate::schema::generic::designation;
use crate::worker::Worker;
use crate::{
    designation::Designation, generic::Generic, model::*, schema::episode as episode_table,
    schema::episode::dsl::episode as episode_db, schema::file_version as file_version_table,
    schema::file_version::dsl::file_version as file_version_data, schema::generic as generic_table,
    schema::generic::dsl::generic as generic_data, schema::show as show_table,
    schema::show::dsl::show as show_db, schema::worker as worker_table,
    schema::worker::dsl::worker as worker_data, show::Episode, show::Show,
};
use diesel::{pg::PgConnection, prelude::*};
use std::collections::VecDeque;
use std::env;
use tracing::{debug, error};

///Sets up a connection to the database via DATABASE_URL environment variable
pub fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|err| {
        error!("DATABASE_URL must be set. Err: {}", err);
        panic!();
    });
    PgConnection::establish(&database_url).unwrap_or_else(|err| {
        error!("Error connecting to {}. Err: {}", database_url, err);
        panic!();
    })
}

pub fn update_file_version(file_version: &FileVersion) {
    let file_version_model = FileVersionModel::from_file_version(file_version);
    match diesel::update(file_version_data).set(&file_version_model).execute(&establish_connection()) {
        Err(err) => {
            error!("Something oopsied with the database. {}", err);
            panic!();
        },
        Ok(_) => {
            //Do nothing right now
            //TODO: Make this do something
        },
    }
}

///Inserts generic data into the database
pub fn create_generics(conn: &PgConnection, new_generics: Vec<NewGeneric>) -> Vec<GenericModel> {
    diesel::insert_into(generic_table::table)
        .values(&new_generics)
        .get_results(conn)
        .unwrap_or_else(|err| {
            error!("Error saving new generics. Err: {}", err);
            panic!();
        })
}

///Inserts file_version data into the database
pub fn create_file_versions(
    connection: &PgConnection,
    new_file_versions: Vec<NewFileVersion>,
) -> Vec<FileVersionModel> {
    diesel::insert_into(file_version_table::table)
        .values(&new_file_versions)
        .get_results(connection)
        .unwrap_or_else(|err| {
            error!("Error saving new file_versions. Err: {}", err);
            panic!();
        })
}

pub fn create_file_version(
    new_file_version: NewFileVersion,
) -> FileVersionModel {
    create_file_versions(&establish_connection(), vec![new_file_version])[0].to_owned()
}

///Inserts show data into the database
pub fn create_show(connection: &PgConnection, show_title: String) -> ShowModel {
    let new_show = NewShow { show_title };

    diesel::insert_into(show_table::table)
        .values(&new_show)
        .get_result(connection)
        .unwrap_or_else(|err| {
            error!("Error saving new show. Err: {}", err);
            panic!();
        })
}

///Inserts episode data into the database
pub fn create_episodes(conn: &PgConnection, new_episode: Vec<NewEpisode>) -> Vec<EpisodeModel> {
    diesel::insert_into(episode_table::table)
        .values(&new_episode)
        .get_results(conn)
        .unwrap_or_else(|err| {
            error!("Error saving new episode. Err: {}", err);
            panic!();
        })
}

pub fn create_worker(conn: &PgConnection, new_worker: NewWorker) -> i32 {
    let worker: WorkerModel = diesel::insert_into(worker_table::table)
        .values(&new_worker)
        .get_result(conn)
        .unwrap_or_else(|err| {
            error!("Error saving new worker. Err: {}", err);
            panic!();
        });
    worker.id
}

pub fn print_all_worker_models() {
    for worker_model in worker_data
        .load::<WorkerModel>(&establish_connection())
        .unwrap_or_else(|err| {
            error!("Error loading worker. Err: {}", err);
            panic!();
        })
    {
        debug!(
            "UID: {}, Last known IP address: {}",
            worker_model.id, worker_model.worker_ip_address
        );
    }
}

pub fn get_all_file_versions() -> Vec<FileVersion> {
    let connection = establish_connection();

    let file_version_models = file_version_data
        .load::<FileVersionModel>(&connection)
        .unwrap_or_else(|err| {
            error!("Error loading file_version. Err: {}", err);
            panic!();
        });

    let mut file_versions: Vec<FileVersion> = Vec::new();
    for file_version in file_version_models {
        file_versions.push(FileVersion::from_file_version_model(file_version));
    }
    file_versions
}

///Get all generic from the database
pub fn get_all_generics() -> Vec<Generic> {
    let connection = establish_connection();

    let generic_models = generic_data
        .filter(designation.eq(Designation::Generic as i32))
        .load::<GenericModel>(&connection)
        .unwrap_or_else(|err| {
            error!("Error loading generic. Err: {}", err);
            panic!();
        });

    let mut generics: Vec<Generic> = Vec::new();
    for generic_model in generic_models {
        generics.push(Generic::from_generic_model(generic_model));
    }
    generics
}

pub fn worker_exists(uid: i32) -> bool {
    for worker in worker_data
        .load::<WorkerModel>(&establish_connection())
        .unwrap_or_else(|err| {
            error!("Error loading worker. Err: {}", err);
            panic!();
        })
    {
        if worker.id == uid {
            return true;
        }
    }
    false
}

pub fn get_all_workers() -> VecDeque<Worker> {
    let connection = establish_connection();

    let worker_models = worker_data
        .load::<WorkerModel>(&connection)
        .unwrap_or_else(|err| {
            error!("Error loading generic. Err: {}", err);
            panic!();
        });

    let mut workers: VecDeque<Worker> = VecDeque::new();
    for worker_model in worker_models {
        workers.push_back(Worker::from_worker_model(worker_model));
    }
    workers
}

pub fn get_all_shows() -> Vec<Show> {
    let connection = establish_connection();
    let raw_shows = show_db
        .load::<ShowModel>(&connection)
        .unwrap_or_else(|err| {
            error!("Error loading shows. Err: {}", err);
            panic!();
        });

    //these all contain the episode designation
    let generic_models = generic_data
        .filter(designation.eq(Designation::Episode as i32))
        .load::<GenericModel>(&connection)
        .unwrap_or_else(|err| {
            error!("Error loading generic. Err: {}", err);
            panic!();
        });

    let mut generics: Vec<Generic> = Vec::new();
    for generic_model in generic_models {
        generics.push(Generic::from_generic_model(generic_model));
    }

    let episode_models = episode_db
        .load::<EpisodeModel>(&connection)
        .unwrap_or_else(|err| {
            error!("Error loading episodes. Err: {}", err);
            panic!();
        });
    let mut episodes: Vec<Episode> = Vec::new();

    for episode_model in episode_models {
        for generic in &generics {
            if generic.get_generic_uid() == episode_model.generic_uid {
                let episode = Episode::new(
                    generic.clone(),
                    episode_model.show_uid,
                    "".to_string(),
                    episode_model.season_number,
                    vec![episode_model.episode_number],
                ); //temporary first episode_number
                episodes.push(episode);
                break;
            }
        }
    }

    let mut shows: Vec<Show> = Vec::new();
    for show in raw_shows {
        shows.push(Show::from_show_model(show));
    }
    for episode in episodes {
        let show_uid = episode.show_uid;
        for show in &mut shows {
            if show.show_uid == show_uid {
                show.insert_episode(episode);
                break;
            }
        }
    }
    shows
}
