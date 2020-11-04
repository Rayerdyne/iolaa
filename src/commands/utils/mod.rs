use std::collections::{HashMap, HashSet};
use std::fmt;
use std::str::FromStr;

pub struct FolderSet {
    folders: HashSet<String>,
    urls: HashMap<String, String>,
}

impl FolderSet {
    pub fn new() -> Self {
        Self {
            folders: HashSet::new(),
            urls: HashMap::new()
        }
    }

    pub fn add_folder(&mut self, name: &str) {
        self.folders.insert(String::from(name));
    }

    pub fn set_in_folder(&mut self, folder: &str, name: &str, value: &str) -> Option<()> {
        let s = String::from(format!("{}_{}", folder, name));
        self.urls.insert(s, String::from(value));
        Some(())
    }

    /// Returns `true` if the `FolderSet` is empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.folders.len() == 0
    }

    /// Returns `true` if this `FolderSet` contains the folder.
    pub fn contains_folder(&self, folder: &str) -> bool {
        self.folders.contains(folder)
    }

    /// Returns a reference to the value corresponding to the key in the folder.
    #[allow(dead_code)]
    pub fn get(&self, folder: &str, name: &str) -> Option<&String> {
        let s = String::from(format!("{}_{}", folder, name));
        self.urls.get(&s)
    }

    /// Removes one entry in one folder.
    #[allow(dead_code)]
    pub fn remove_entry(&mut self, folder: &str, name: &str) -> Option<String> {
        let s = String::from(format!("{}_{}", folder, name));
        match self.urls.remove_entry(&s) {
            Some((_, v)) => Some(v),
            None => None
        }
    }

    /// Removes a folder and all its entries.
    #[allow(dead_code)]
    pub fn remove_folder(&mut self, folder: &str) -> bool {
        self.urls.retain(|k, _v| !k.starts_with(folder));
        self.folders.remove(&String::from(folder))
    }

    /// Returns a set of entries in a folder.
    #[allow(dead_code)]
    pub fn list_folder(&self, folder: &str) -> HashSet<String> {
        let mut res: HashSet<String> = HashSet::new();
        for (k, _v) in self.urls.clone() {
            if k.starts_with(folder) {
                res.insert(String::from(k.split("_").nth(1).unwrap()));
            }
        }
        res
    }
}

impl fmt::Display for FolderSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut res = String::new();
        for folder_name in self.folders.clone() {
            res.push_str(&format!("\"{}\": {{\n", folder_name));

            for (name, url) in &self.urls {
                if name.starts_with(&folder_name) {
                    let short_name = name.split("_").nth(1).unwrap();
                    res.push_str(&format!("\t\"{}\": \"{}\"\n", short_name, url));
                }
            }
            res.push_str("},\n");
        }

        write!(f, "{}", res)
    }
}

impl FromStr for FolderSet {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut fs = FolderSet::new();
        let folders : Vec<&str> = s.split(',').collect();
        for k in 0..folders.len()-1 {
            let folder = folders[k];
            let lines : Vec<&str> = folder.trim().split('\n').collect();
            let l0_parts : Vec<&str> = lines[0].split('"')
                                               .filter(|s| !s.is_empty())
                                               .collect();
            let folder_name = l0_parts[0].trim();
            fs.add_folder(&String::from(folder_name));

            for i in 1..lines.len()-1 {
                println!("line of {}: {}", folder_name, lines[i]);
                let line_parts : Vec<&str> = lines[i].split('"').collect();
                let name = line_parts[1].trim();
                let url = line_parts[3].trim();
                fs.set_in_folder( &String::from(folder_name), 
                                  &String::from(name),
                                  &String::from(url));
            }
        }

        Ok(fs)
    }
}