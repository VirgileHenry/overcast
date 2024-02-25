pub(crate) mod networking;
pub(crate) mod config;

use config::ClientConfig;
use networking::network_manager::NetworkManager;
use overcast_core::log::{self, Logger};


fn create_resources(config: &ClientConfig, logger: &Logger) -> Result<(
    NetworkManager,
), Box<dyn std::error::Error>> {

    let network_manager = NetworkManager::new(config, logger.clone())?;

    
    Ok((
        network_manager,
    ))
}


fn main() {

    let config = ClientConfig::default();

    // for development, we can redirect them to the stdout ?
    let _log_file = std::fs::File::create("../logs.log").unwrap();
    let stdout = std::io::stdout();
    #[cfg(debug_assertions)]
    let log_level = log::LogLevel::Debug;
    #[cfg(not(debug_assertions))]
    let log_level = log::LogLevel::Info;
    let (log_manager, logger) = overcast_core::log::LogManager::new(stdout, log_level);
    let _ = log_manager.start();
    logger.log("Starting overcaster client", overcast_core::log::LogLevel::Info);

    match create_resources(&config, &logger) {
        Ok((
            network_manager,
        )) => {
            println!("Starting bevy");
            bevy::prelude::App::new()
                .add_plugins(bevy::prelude::DefaultPlugins)
                .insert_resource(network_manager)
                .add_systems(bevy::prelude::Update, NetworkManager::recv_update)
                .run();
        }
        Err(e) => {
            logger.log(&format!("Unable to create resources: {e} unable to create client!"), log::LogLevel::Critical);
        }
    }


}
