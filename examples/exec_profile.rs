use cct::{config, launch};
use std::env;

fn main() {
    let toml_path = env::var("CCT_TEST_TOML")
        .expect("CCT_TEST_TOML must be set");
    env::set_var("CCT_CONFIG", &toml_path);
    let profiles = config::load_profiles().expect("load profiles");
    let profile = profiles.into_iter().next().expect("at least one profile");
    launch::restore_terminal();
    let err = launch::exec_claude(&profile);
    eprintln!("exec_profile: {err:#}");
    std::process::exit(1);
}
