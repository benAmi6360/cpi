use std::io::Error;
use std::fs;
use std::collections::HashSet;
use std::path::{PathBuf, Path};

use crate::local::Local;
use crate::version::Version;

use crate::toml::{PackageManifest, Dependency, TomlError};
use super::ModuleError;

const PATH: &str = "/opt/clibs";
const PACKAGE_METADATA: &str = "metadata.toml";

#[derive(Debug, Clone)]
pub struct Module {
    pub dependencies: HashSet<Dependency>,
    pub version: Version,
    pub name: String,
    pub path: PathBuf
}

impl Module {
    pub fn new(name: String) -> Result<Module, ModuleError> {
        let path: PathBuf = format!("{}/{}", PATH, name).into();
        if !path.is_dir() {
            return Err(ModuleError::ModuleNotExist);
        }
        let path_to_toml = format!("{}/metadata.toml", path.display());
        let manifest_content = fs::read_to_string(path_to_toml)?;
        let pkg: PackageManifest = toml::from_str(&manifest_content).unwrap();
        Ok(
            Module {
                    dependencies: HashSet::from_iter(pkg.dependencies.unwrap_or(Vec::new()).into_iter()),
                    version: Version::try_from(pkg.version)?,
                    name: pkg.title,
                    path: path
            }
        )
    }

    pub fn install(&self) -> Result<(), ModuleError> {
        
        match Local::is_installed(self) {
            Ok(boolean) => {
                if boolean {
                    println!("Installed already");
                    return Ok(());
                }
            }
            Err(e) => return Err(e.into())
        }
        self.install_dependencies()?;
        println!("Downloading {}", &self.name);
        let header = format!("{}/{}.h", self.path.display(), self.name);
        fs::copy(
            header, 
            format!("./build/libs/headers/{}.h", self.name)
        )?;
        let shared = format!("{}/{}.so", self.path.display(), self. name);
        fs::copy(
            shared, 
            format!("./build/libs/objs/{}.so", self.name)
        )?;

        match Local::add(self) {
            Ok(_) => Ok(()),
            Err(e) => {
                println!("{:?}", e);
                self.uninstall()?;
                Err(e.into())
            }
        }
    }


    /// # Panics
    pub fn uninstall(&self) -> Result<(), TomlError> {
        match Local::is_installed(self) {
            Ok(boolean) => {
                if !boolean {
                    println!("The package {} is not installed", self.name);
                    return Ok(())
                }
            },
            Err(e) => return Err(e)
        }
        fs::remove_file(Path::new(&format!("./build/libs/objs/{}.so", self.name)))?;
        fs::remove_file(Path::new(&format!("./build/libs/headers/{}.h", self.name)))?;

        Local::remove(self)?;

        println!("Removed {}", self.name);
        Ok(())
    }
    
    pub fn get_info(&self) -> std::io::Result<String> {
        let mut path = self.path.clone();
        
        path.push(PACKAGE_METADATA);
        fs::read_to_string(&path)
    }
    fn install_dependencies(&self) -> Result<(), Error>{
        self.dependencies.iter().for_each(|f| f.install().unwrap());
        Ok(())
    }
    
    pub fn get_list_of_all_modules() -> Vec<Module> {
        let path: PathBuf = PATH.into();
        match fs::read_dir(path) {
            Ok(entries) => {
                entries
                .filter_map(|entry| 
                    entry.ok()
                    .and_then(|entry| 
                        entry
                            .file_name()
                            .into_string()
                            .ok()))
                .filter_map(|it| Module::new(it).ok())
                .collect()
            }
            Err(_) => {
                panic!("BRUH WE FUCKED UP");
            }
        }
        
    }
}

impl From<&Module> for Dependency {
    fn from(module: &Module) -> Dependency {
        Dependency {
            name: module.name.to_owned(),
            version: module.version.to_string()
        }
    }
}

impl PartialEq for Module {
    fn eq(&self, other: &Self) -> bool {
        return self.name == other.name && self.version == other.version;
    }
}

impl Eq for Module {}