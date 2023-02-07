use std::{
    net::{IpAddr, Ipv6Addr},
    path::PathBuf,
    sync::Arc,
};

pub type ArcConfig = Arc<Config>;

#[derive(Debug)]
pub struct Config {
    pub addr: IpAddr,
    pub port: u16,
    pub assets_dir: PathBuf,
}

impl Config {
    pub fn show_help() {
        println!("HELP:\n{}", env::gen_help());
    }

    pub fn from_env() -> Self {
        env::assert_env_vars();

        Self {
            addr: env::addr().unwrap_or(IpAddr::V6(Ipv6Addr::LOCALHOST)),
            port: env::port().unwrap_or(8888),
            assets_dir: env::assets_dir().unwrap_or_else(|| PathBuf::from("./assets/")),
        }
    }
}

mod env {
    use super::*;

    menv::require_envs! {
        (assert_env_vars, any_set, gen_help);

        addr?, "PECULIARZONE_BINDING_ADDR", IpAddr,
        "PECULIARZONE_BINDING_ADDR: Listener binding address";

        port?, "PECULIARZONE_PORT", u16,
        "PECULIARZONE_PORT: Listener binding port";

        assets_dir?, "PECULIARZONE_ASSETS_DIR", PathBuf,
        "PECULIARZONE_ASSETS_DIR: Directory where assets are to be found";
    }
}
