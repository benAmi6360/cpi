use std::io::Error;
use std::fs;
use std::hash::{Hash, Hasher};
use std::collections::HashSet;
use crate::utils::*;
use crate::version::{Version, VersionOptions};
use super::ModuleError;

#[derive(Debug, Clone)]
pub struct Module {
    pub dependencies: HashSet<Module>,
    pub version: Version,
    pub name: String,
    pub path: String
}

impl Module {
    pub fn new(name: String) -> Result<Module, ModuleError> {
        let path = format!("{}/{}", PATH, name);
        let dependencies_path = format!("{}/dependencies", path);
        let dependencies: HashSet<Module> = 
            read_subdirs_to_collection::<HashSet<String>>(dependencies_path.as_str())?
            .into_iter()
            .map(|it| Module::new(it).unwrap())
            .collect();
        let version: Version = get_version_of_module(&path)?;
        let module = Module {
            version: version,
            name: name,
            path: path,
            dependencies: dependencies
        };
        
        if does_module_exist(&module).is_err() {
            return Err(ModuleError::ModuleNotExist);
        }
        Ok(module)
    }

    pub fn install(&self) -> Result<(), Error> {
        let _ = self.install_dependencies()?;

        println!("Downloading {}", &self.name);
        let header = format!("{}/{}.h", self.path, self.name);
        let _ = fs::copy(
            header, 
            format!("./build/libs/headers/{}.h", self.name)
        )?;
        let shared = format!("{}/{}.so", self.path, self. name);
        let _ = fs::copy(
            shared, 
            format!("./build/libs/objs/{}.so", self.name)
        )?;

        Ok(())
    }
    
    fn install_dependencies(&self) -> Result<(), Error>{
        if self.dependencies.is_empty() {
            return Ok(());
        }
        self.dependencies.iter().for_each(|f| f.install().unwrap());
        Ok(())
    }

    pub fn update(&mut self, opt: VersionOptions) {
        self.version.increment(opt);
    }
}

impl Hash for Module {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for Module {
    fn eq(&self, other: &Self) -> bool {
        return self.name == other.name;
    }
}

impl Eq for Module {}