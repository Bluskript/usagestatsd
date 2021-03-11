use comms::IPC;
use crossbeam_channel::{bounded, select, Receiver};
use monitor::{Monitor, ProcessHandler};
use package_backend::alpm_backend::AlpmBackend;
use std::{
    sync::{Arc, Mutex},
    thread,
};
use store::Store;
use tracing::{error, subscriber::set_global_default, Level};
use tracing_subscriber::FmtSubscriber;

pub mod comms;
pub mod monitor;
pub mod package_backend;
pub mod store;

fn ctrl_channel() -> Result<Receiver<()>, ctrlc::Error> {
    let (sender, receiver) = bounded(100);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })?;

    Ok(receiver)
}

fn setup_tracing() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .pretty()
        .finish();
    set_global_default(subscriber)?;
    Ok(())
}

#[tokio::main]
pub async fn main() {
    setup_tracing().unwrap();
    init_daemon().await;
}

async fn init_daemon() {
    let ctrl_c_channel = ctrl_channel().unwrap();
    let store = Arc::new(Mutex::new(Store::new().unwrap()));
    let alpm_backend = AlpmBackend::new("/", "/var/lib/pacman").unwrap();
    let process_handler = Arc::new(Mutex::new(
        ProcessHandler::new(Box::new(alpm_backend), store.clone()).unwrap(),
    ));
    let mut monitor = Monitor::new(process_handler.clone()).unwrap();

    thread::spawn(move || monitor.event_reader());

    match IPC::new("me.blusk.usagestatsd", store).await {
        Err(e) => error!("{:?}", e),
        _ => (),
    };

    loop {
        select! {
            recv(ctrl_c_channel) -> _ => {
                break;
            }
        }
    }
}
