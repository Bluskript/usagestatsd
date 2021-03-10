use crate::{package_backend::PackageBackend, store::Store};
use cnproc::PidMonitor;
use procfs::process::Process;
use std::collections::HashSet;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tracing::error;

pub struct Monitor {
    cn_proc_sock: PidMonitor,
    process_handler: Arc<Mutex<ProcessHandler>>,
}

pub struct ProcessHandler {
    store: Store,
    file_owners: HashMap<String, String>,
    used_packages: HashSet<String>,
    package_backend: Box<dyn PackageBackend + Send>,
}

impl Monitor {
    pub fn new(
        process_handler: Arc<Mutex<ProcessHandler>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Monitor {
            cn_proc_sock: PidMonitor::new()?,
            process_handler: process_handler,
        })
    }

    pub fn event_reader(&mut self) {
        loop {
            if let Some(ev) = self.cn_proc_sock.recv() {
                match ev {
                    cnproc::PidEvent::Exec(pid) | cnproc::PidEvent::Fork(pid) => {
                        match self.process_handler.lock().unwrap().on_process_start(pid) {
                            Err(error) => error!("{:?}", error),
                            _ => (),
                        }
                    }
                    _ => (),
                }
            }
        }
    }
}

impl ProcessHandler {
    pub fn new(
        package_backend: Box<dyn PackageBackend + Send>,
        store: Store,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(ProcessHandler {
            file_owners: HashMap::new(),
            used_packages: HashSet::new(),
            store,
            package_backend,
        })
    }

    fn on_process_start(&mut self, pid: i32) -> Result<(), Box<dyn std::error::Error>> {
        let proc = Process::new(pid)?;
        let path = proc.exe()?;
        let mut path_str = path.to_str().ok_or("error getting path string")?;
        path_str = path_str.strip_prefix("/").unwrap();
        if !self.file_owners.contains_key(path_str) {
            self.file_owners.insert(
                path_str.to_string(),
                self.package_backend
                    .as_ref()
                    .get_file_owner_pkg(path_str)?
                    .to_string(),
            );
        }
        if let Some(owner) = self.file_owners.get(path_str) {
            match self.store.update_last_opened(owner) {
                Err(err) => error!("error updating DB: {:?}", err),
                _ => (),
            }
            self.used_packages.insert(owner.clone());
        }
        Ok(())
    }
}
