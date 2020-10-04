use alpm::Error;
use procfs::process::Process;
use std::collections::HashMap;
use std::collections::HashSet;

pub struct Handler {
    alpm: alpm::Alpm,
    file_owners: HashMap<String, String>,
    used_packages: HashSet<String>,
}

impl Handler {
    pub fn new() -> Result<Handler, Error> {
        return Ok(Handler {
            file_owners: HashMap::new(),
            used_packages: HashSet::new(),
            alpm: alpm::Alpm::new("/", "/var/lib/pacman/")?,
        });
    }

    pub fn on_process_start(&mut self, pid: i32) -> Result<(), Box<dyn std::error::Error>> {
        let proc = Process::new(pid)?;
        let path = proc.exe()?;
        let mut path_str = path.to_str().ok_or("error getting path string")?;
        path_str = path_str.strip_prefix("/").unwrap();
        if !self.file_owners.contains_key(path_str) {
            self.file_owners
                .insert(path_str.to_string(), self.get_file_owner_pkg(path_str)?);
        }
        if let Some(owner) = self.file_owners.get(path_str) {
            self.used_packages.insert(owner.clone());
        }
        return Ok(());
    }

    pub fn get_file_owner_pkg(&self, path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let db = self.alpm.localdb();
        let pkgs = db.pkgs()?;
        for pkg in pkgs {
            let res = pkg.files().contains(path)?;
            if res.is_some() {
                return Ok(pkg.name().to_string());
            }
        }
        return Err("nobody owns the file".into());
    }
}
