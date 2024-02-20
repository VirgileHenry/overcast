use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use bevy::{app::AppExit, ecs::event::EventWriter, prelude::Resource};
use overcast_core::log::{LogLevel, Logger};

#[derive(Resource)]
pub(crate) struct ControlFlowHandler {
    exit_signal_received: Arc<AtomicBool>,
    logger: Logger,
}

impl ControlFlowHandler {
    pub(crate) fn new(logger: Logger) -> ControlFlowHandler {
        let exit_signal_received = Arc::new(AtomicBool::new(false));
        let ctrl_c_signal = exit_signal_received.clone();
        if let Err(e) = ctrlc::set_handler(move || {
            ctrl_c_signal.store(true, Ordering::SeqCst);
        }) {
            logger.log(&format!("Unable to set SIGINT handler: {}", e), LogLevel::Warning);
        }
        ControlFlowHandler { 
            exit_signal_received,
            logger,
        }
    }
}

pub(crate) fn checks_control_flow(control_flow_handler: bevy::prelude::Res<ControlFlowHandler>, mut exit: EventWriter<AppExit>) {
    if control_flow_handler.exit_signal_received.load(Ordering::SeqCst) {
        control_flow_handler.logger.log("Signal SIGINT received, stopping bevy.", LogLevel::Info);
        exit.send(AppExit);
    }
}