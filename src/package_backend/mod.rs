use std::error::Error;

pub mod alpm_backend;

pub trait PackageBackend {
    fn get_file_owner_pkg(&self, path: &str) -> Result<&str, Box<dyn Error>>;
}
