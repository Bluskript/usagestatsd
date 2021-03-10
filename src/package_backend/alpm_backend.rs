use std::error::Error;

use alpm::Alpm;

use super::PackageBackend;

pub struct AlpmBackend {
    alpm: Alpm,
}

impl AlpmBackend {
    /// Returns a new instance of AlpmBackend
    /// # Arguments
    /// * `root` the root directory
    /// * `path` the path where pacman is located
    pub fn new(root: &str, path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(AlpmBackend {
            alpm: Alpm::new("/", "/var/lib/pacman/")?,
        })
    }
}

impl PackageBackend for AlpmBackend {
    fn get_file_owner_pkg(&self, path: &str) -> Result<&str, Box<dyn Error>> {
        let db = self.alpm.localdb();
        let pkgs = db.pkgs()?;
        for pkg in pkgs {
            let res = pkg.files().contains(path)?;
            if res.is_some() {
                return Ok(pkg.name());
            }
        }
        Err("nobody owns the file".into())
    }
}
