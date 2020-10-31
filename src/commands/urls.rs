use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use std::fs::File;
use std::io::{Read, Error, ErrorKind};

struct FolderSet {
    folders: HashMap<String, HashMap<String, String>>
}

impl FolderSet {
    fn new() -> FolderSet {
        FolderSet {
            folders: HashMap::new()
        }
    }

    fn add_folder(&mut self, name: &str) {
        let f = HashMap::new();
        self.folders.insert(String::from(name), f);
    }

    fn set_in_folder(&mut self, folder: &str, name: &str, value: &str) -> Option<()> {
        let f = self.folders.get_mut(folder)?;
        f.insert(String::from(name), String::from(value));
        Some(())
    }
}

impl fmt::Display for FolderSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut res = String::new();
        for (folder_name, folder) in self.folders.clone() {
            res.push_str(format!("\"{}\": {{\n", folder_name).as_str());
            for (name, url) in folder {
                res.push_str(format!("\t\"{}\": \"{}\"\n", name, url).as_str());
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
            fs.add_folder(String::from(folder_name).as_str());

            for i in 1..lines.len()-1 {
                println!("line of {}: {}", folder_name, lines[i]);
                let line_parts : Vec<&str> = lines[i].split('"').collect();
                let name = line_parts[1].trim();
                let url = line_parts[3].trim();
                fs.set_in_folder( String::from(folder_name).as_str(), 
                                  String::from(name).as_str(),
                                  String::from(url).as_str());
            }
        }

        Ok(fs)
    }
}

#[allow(dead_code)]
fn load_urls(filename: &str) -> Result<FolderSet, Error> {
    let mut file = File::open(filename)?;
    let mut data = String::new();
    if let Err(e) = file.read_to_string(&mut data) {
        return Err(e);
    }
    
    match FolderSet::from_str(&mut data) {
        Ok(fs) => Ok(fs),
        Err(s) => Err(Error::new(ErrorKind::Other, s))
    }
}


#[test]
fn test_folders() {
    let mut fs = FolderSet::new();
    fs.add_folder("bonjour");
    fs.set_in_folder("bonjour", "var1", "url1");
    fs.set_in_folder("bonjour", "var2", "url2");
    fs.add_folder("caca");
    fs.set_in_folder("caca", "varname1", "urlname1");
    println!("{}", fs); 
}

#[test]
fn test_from_str_folder() {
    let mut fs = FolderSet::new();
    fs.add_folder("bonjour");
    fs.set_in_folder("bonjour", "var1", "url1");
    fs.set_in_folder("bonjour", "var2", "url2");
    fs.add_folder("caca");
    fs.set_in_folder("caca", "varname1", "urlname1");
    let s = format!("{}", fs);
    println!("s: `{}`", fs);
    match FolderSet::from_str(s.as_str()) {
        Ok(f2) => caca(f2),
        Err(why) => println!("Got an error, {}", why) 
    }
}

fn caca(f: FolderSet) {
    println!("caca-> \n{} ", f);
}

