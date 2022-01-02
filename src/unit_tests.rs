use std::sync::{Arc, RwLock};

use tracing::{debug, error, warn, info};

use crate::{
    config::ServerConfig, pathbuf_copy, pathbuf_create_file, pathbuf_remove_file, pathbuf_to_string,
};

//Default "Pass" status
#[derive(PartialEq)]
pub enum Status {
    Pass,
    PassWithWarnings,
    Fail,
}

impl Status {
    #[allow(clippy::inherent_to_string)]
    fn to_string(&self) -> String {
        match self {
            Self::Pass => "Pass".to_string(),
            Self::PassWithWarnings => "Pass with warnings".to_string(),
            Self::Fail => "Fail".to_string(),
        }
    }
}

pub struct UnitTest {
    test_name: String,
    status: Status,
    warnings: u16,
    //Used to weed out any count from a consecutive failure
    definitive_errors: u16,
}

impl UnitTest {
    pub fn new(test_name: String) -> Self {
        info!("{} self test started.", test_name);
        Self {
            test_name,
            status: Status::Pass,
            warnings: 0,
            definitive_errors: 0,
        }
    }
    pub fn add_warning(&mut self) {
        self.warnings += 1;
        if self.status == Status::Pass {
            self.status = Status::PassWithWarnings;
        }
    }

    pub fn definitive_error_occurred(&mut self) {
        self.definitive_errors += 1;
        if self.status != Status::Fail {
            self.status = Status::Fail;
        }
    }

    pub fn output_result(&self) {
        let text = format!("{} self test finished. Status: {}", self.test_name, self.status.to_string());
        match self.status {
            Status::Pass => info!("{}", text),
            Status::PassWithWarnings => warn!("{}", text),
            Status::Fail => error!("{}", text),
        }
    }
}

//Self tests
//No panics
//Will try to guarantee causes, or provide the most likely, but still possible causes

//TODO: Call pathbuf modifying functions with a given output and expected return, can utilise asserteq if it doesn't force a panic!()
//Server temp
//Server cache
//Ensure workers have read/write access to the network/local share temp (check from all active worker's context and the server)
//Ensure the worker only has read-only access to the main library

//TODO: Check if the config file is valid and if not, can it be recovered from a default config, display the discrepancy
//    : This is an example of potential PassWithWarnings

//File access self test
//I only care about explicit failures
//This will output debug text, but will just return a pass/fail
pub fn file_access_self_test(server_config: Arc<RwLock<ServerConfig>>) -> bool {
    let test_file_name = ".unit_test_file.txt";//Hidden file

    //TODO: Run cleanup before starting self-tests (removing annotated test files that might exist on the system)

    //Test paths
    let server_cache_directory;
    let global_temp_directory;
    let root_directories;
    {
        let server_config_lock_tracked_directories =
            &server_config.read().unwrap().tracked_directories;
        server_cache_directory = server_config_lock_tracked_directories
            .get_cache_directory()
            .clone();
        global_temp_directory = server_config_lock_tracked_directories
            .get_global_temp_directory()
            .clone();
        root_directories = server_config_lock_tracked_directories.get_root_directories();
    }

    //Server
    {
        let mut server_test = UnitTest::new("Server access".to_string());
        let server_cache_test_file_path = server_cache_directory.join(test_file_name);
        info!("\"server_cache_test_file_path\": {}", pathbuf_to_string(&server_cache_test_file_path));
        let global_temp_test_file_path = global_temp_directory.join(test_file_name);
        info!("\"global_temp_test_file_path\": {}", pathbuf_to_string(&global_temp_test_file_path));

        //Create test file in server cache
        let mut create_test_file_error: bool = false;
        if let Err(err) = pathbuf_create_file(&server_cache_test_file_path) {
            error!(
                "Create test file in server cache:: \"server_cache_directory\": {} caused error: {}",
                pathbuf_to_string(&server_cache_directory),
                err
            );
            create_test_file_error = true;
            server_test.definitive_error_occurred();
        } else {
            info!("Create test file in server cache:: Successfully created file: {}", pathbuf_to_string(&server_cache_test_file_path));
        }

        //Copy test file from server cache to global temp
        let mut copy_test_file_destination_error = false;
        if let Err(err) = pathbuf_copy(&server_cache_test_file_path, &global_temp_test_file_path) {
            if create_test_file_error {
                warn!("Copy test file from server cache to global temp:: \"server_cache_directory\": {} caused this consecutive failure, but \"global_temp_directory\": {} could also cause this error in a future run: {}", pathbuf_to_string(&server_cache_directory), pathbuf_to_string(&global_temp_directory), err);
            } else {
                error!(
                    "Copy test file from server cache to global temp:: \"global_temp_directory\": {} caused error: {}",
                    pathbuf_to_string(&global_temp_directory),
                    err
                );
                copy_test_file_destination_error = true;
                server_test.definitive_error_occurred();
            }
        } else {
            info!("Copy test file from server cache to global temp:: Successfully copied file from {} to {}", pathbuf_to_string(&server_cache_test_file_path), pathbuf_to_string(&global_temp_test_file_path));
        }

        //Remove test file from server cache
        if let Err(err) = pathbuf_remove_file(&server_cache_test_file_path) {
            if create_test_file_error {
                warn!(
                    "Remove test file from server cache:: \"server_cache_directory\": {} caused this consecutive failure: {}",
                    pathbuf_to_string(&server_cache_directory),
                    err
                );
            } else {
                error!(
                    "Remove test file from server cache:: \"server_cache_directory\": {} caused error: {}",
                    pathbuf_to_string(&server_cache_directory),
                    err
                );
                server_test.definitive_error_occurred();
            }
        } else {
            info!("Remove test file from server cache:: Successfully removed file: {}", pathbuf_to_string(&server_cache_test_file_path));
        }

        //Remove file from global temp
        if let Err(err) = pathbuf_remove_file(&global_temp_test_file_path) {
            if copy_test_file_destination_error {
                warn!(
                    "Remove file from global temp::\"global_temp_directory\": {} caused this consecutive failure: {}",
                    pathbuf_to_string(&global_temp_directory),
                    err
                );
            } else if create_test_file_error {
                warn!("Remove file from global temp::\"server_cache_directory\": {} caused this consecutive failure, but \"global_temp_directory\": {} could also cause this error in a future run: {}", pathbuf_to_string(&server_cache_directory), pathbuf_to_string(&global_temp_directory), err);
            } else {
                error!(
                    "Remove file from global temp::\"global_temp_directory\": {} caused error: {}",
                    pathbuf_to_string(&global_temp_directory),
                    err
                );
                server_test.definitive_error_occurred();
            }
        } else {
            info!("Remove file from global temp:: Successfully removed file: {}", pathbuf_to_string(&server_cache_test_file_path));
        }

        server_test.output_result();
    }

    //Workers
    //Output error unit failure if no workers are available to test with

    //WebUI

    //pathbuf_create_file
    //pathbuf_copy
    //pathbuf_remove_file
    true
}
