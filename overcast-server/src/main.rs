pub(crate) mod networking;
pub(crate) mod control_flow;
pub(crate) mod config;
pub(crate) mod player;


fn create_resources(logger: &overcast_core::log::Logger, config: &self::config::ServerConfig) -> Result<(
    self::networking::network_manager::NetworkManager,
    self::control_flow::ControlFlowHandler,
), Box<dyn std::error::Error>> {
    Ok((
        self::networking::network_manager::NetworkManager::new(logger.clone(), &config)?,
        self::control_flow::ControlFlowHandler::new(logger.clone()),
    ))
}

fn main() {

    let config = self::config::ServerConfig::default();

    // for development, we can redirect them to the stdout ?
    let _log_file = std::fs::File::create("../logs.log").unwrap();
    let stdout = std::io::stdout();
    let (log_manager, logger) = overcast_core::log::LogManager::new(stdout, overcast_core::log::LogLevel::Debug);
    let _ = log_manager.start();
    logger.log("Welcome, Overcasters!", overcast_core::log::LogLevel::Info);

    match create_resources(&logger, &config) {
        Err(e) => {
            logger.log(&format!("Error while attempting to create resources: {e}"), overcast_core::log::LogLevel::Critical);
        }
        Ok((
            network_manager,
            control_flow_handler,
        )) => {
            bevy::prelude::App::new()
                .add_plugins(bevy::app::ScheduleRunnerPlugin::run_loop(config.frame_delay()))
                .insert_resource(network_manager)
                .insert_resource(control_flow_handler)
                .add_systems(bevy::prelude::PreUpdate, control_flow::checks_control_flow)
                .add_systems(bevy::prelude::Update, networking::network_manager::NetworkManager::recv_update)
                .run();
        }
    }
    logger.log("Bye, Overcasters!", overcast_core::log::LogLevel::Info);
}
