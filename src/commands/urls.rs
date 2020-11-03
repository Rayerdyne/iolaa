use serenity::{
    prelude::*,
    model::prelude::*,
    framework::standard::{
        Args, CommandResult, macros::command
    },
    utils::MessageBuilder,
};
use std::str::FromStr;
use std::fs::File;
use std::io::{Read, Write, Error as IOError, ErrorKind};
use std::sync::Mutex;
use lazy_static::lazy_static;

use super::utils::FolderSet;

const DEF_FOLDER_NAME: &str = "default";
const DATA_FILE: &str = "data/urls.txt";

// Folder: `folder_name`
// Name:   **name**
// Url:    __url__

// What is the best way to memorize some information from one command to another ?
// For example, I want to to something like:
// `set 3.14159` 
// `get` then the bot answers: `3.14159`
// Is `lazy_static` the thing to use or 

lazy_static! {
    static ref URLS: Mutex<FolderSet> = Mutex::new({
        match load_urls(DATA_FILE) {
            Ok(fs) => fs,
            Err(_) => FolderSet::new(),
        }
    });

    static ref CUR_DIR: Mutex<String> = Mutex::new(String::from(DEF_FOLDER_NAME));
}


#[command]
pub async fn whereis(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let ans = MessageBuilder::new()
        .push("The current directory is: ")
        .push_mono(CUR_DIR.lock().unwrap().as_str())
        .push(".").build();
    msg.channel_id.say(&ctx.http, ans).await?;
    Ok(())
}

#[command]
pub async fn cd(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut target = args.single::<String>()?;
    if target == ".." {
        target = String::from(DEF_FOLDER_NAME);
    }
    if !URLS.lock().unwrap().contains_folder(target.as_str()) {
        let ans = MessageBuilder::new()
            .push("😮 There is no ")    .push_mono(target)
            .push(" directory.")        .build();
        msg.channel_id.say(&ctx.http, ans).await?;
    }
    else {
        CUR_DIR.lock().unwrap().replace_range(.., target.as_str());
        let ans = MessageBuilder::new()
            .push("Moved to ")          .push_mono(target)
            .push(" directory 📂!")     .build();
        msg.channel_id.say(&ctx.http, ans).await?;
    }
    Ok(())
}

#[command]
pub async fn mkdir(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let folder = match args.single::<String>() {
        Ok(s) => s,
        Err(_) => String::new()
    };

    if folder.is_empty() {
        msg.channel_id.say(&ctx.http, "🙊 Must provide the a folder'sname !").await?;
    } else {
        URLS.lock().unwrap().add_folder(folder.as_str());
        let ans = MessageBuilder::new()
            .push("👉Created new ")     .push_mono(folder)
            .push(" directory!")        .build();
        msg.channel_id.say(&ctx.http, ans).await?;
    }

    Ok(())
}

#[command]
pub async fn set(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    loop {
        let name = match args.single::<String>() {
            Ok(s) => s,
            Err(_) => break,
        };
        let url = match args.single::<String>() {
            Ok(s) => s,
            Err(_) => break,
        };

        if name.contains("_") {
            msg.channel_id.say(&ctx.http, "Name cannot contain `'_'`.").await?;
            return Ok(())
        }

        // let folder = CUR_DIR.lock().unwrap().clone().as_str();
        URLS.lock().unwrap().set_in_folder(
            CUR_DIR.lock().unwrap().as_str(), &name, &url);
        
        let ans = MessageBuilder::new()
            .push("Added ")           .push_bold(name.as_str())
            .push(" entry in ")       .push_mono(CUR_DIR.lock().unwrap().as_str())
            .push(" directory 👌!")   .build();
        msg.channel_id.say(&ctx.http, ans).await?;
    }
    Ok(())
}

#[command]
pub async fn get(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    loop {
        let name = match args.single::<String>() {
            Ok(s) => s,
            Err(_) => break,
        };

        let entry = match URLS.lock().unwrap().get(CUR_DIR.lock().unwrap().as_str(), &name) {
            Some(s) => s.clone(),
            None => String::new(),
        };

        if entry.is_empty() {
            let ans = MessageBuilder::new()
                .push("Did not found an entry for ")    .push_bold(name)
                .push(" in folder ")                    .push_mono(CUR_DIR.lock().unwrap())
                .push(" 😮.")                            .build();
            msg.channel_id.say(&ctx.http, ans).await?;
        }
        else {
            let ans = MessageBuilder::new()
                .push_bold(name)                        .push(" (in folder ")
                .push_mono(CUR_DIR.lock().unwrap())     .push(") is ")
                .push_underline(entry)                  .push(".")
                .build();
            msg.channel_id.say(&ctx.http, ans).await?;
        }
    }
    Ok(())
}

#[command]
pub async fn rm(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    loop {
        let name = match args.single::<String>() {
            Ok(s) => s,
            Err(_) => break,
        };

        let entry = match URLS.lock().unwrap().get(CUR_DIR.lock().unwrap().as_str(), name.as_str()) {
            Some(s) => s.clone(),
            None => String::new(),
        };

        if entry.is_empty() {
            let ans = MessageBuilder::new()
                .push("Did not found an entry for ")    .push_bold(name)
                .push(" in folder ")                    .push_mono(CUR_DIR.lock().unwrap())
                .push(" 😮.")                            .build();
            msg.channel_id.say(&ctx.http, ans).await?;
        }
        else {
            let res = match URLS.lock().unwrap().remove_entry(CUR_DIR.lock().unwrap().as_str(), name.as_str()) {
                Some(_) => "Oké !",
                None => "Not oké :-<",
            };
            println!("Remove entry: {}", res);
            let ans = MessageBuilder::new()
                .push_bold(name)                        .push(" has been removed from ")
                .push_mono(CUR_DIR.lock().unwrap())     .push(" folder ✅.")
                .build();
            msg.channel_id.say(&ctx.http, ans).await?;
        }
    }
    Ok(())
}

#[command]
pub async fn rmdir(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let folder = match args.single::<String>() {
        Ok(s) => s,
        Err(_) => String::new()
    };

    if folder.is_empty() {
        msg.channel_id.say(&ctx.http, "🙊 Must provide a folder's name !").await?;
    } else {
        URLS.lock().unwrap().remove_folder(folder.as_str());
        let ans = MessageBuilder::new()
            .push("👉Created new ")     .push_mono(folder)
            .push(" directory!")        .build();
        msg.channel_id.say(&ctx.http, ans).await?;
    }
    Ok(())
}

#[command]
pub async fn ls(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    
    let folder = if args.len() > 0 {
       args.single::<String>()?    
    } else {
        CUR_DIR.lock().unwrap().clone()
    };

    let entries = URLS.lock().unwrap().list_folder(folder.as_str());
    let mut ans = MessageBuilder::new();
    ans.push("Found following entries in folder ")
       .push_mono(folder)   .push(": \n");
    for entry in entries {
        ans.push_mono("- ").push_bold(entry.as_str()).push(",\n");
    }

    msg.channel_id.say(&ctx.http, ans.build()).await?;
    Ok(())
}

#[command]
pub async fn save(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {

    msg.channel_id.say(&ctx.http, format!("Before: {}", URLS.lock().unwrap()).as_str()).await?;
    #[allow(unused_must_use)]
    match save_raw(DATA_FILE, format!("{}", URLS.lock().unwrap()).as_bytes()) {
        Ok(()) => {
            msg.channel_id.say(&ctx.http, "Successfully saved 😘!").await?;
        },
        Err(_) => { 
            msg.channel_id.say(&ctx.http, "Could not write file, some error occured 😕.").await?;
            ()
        }
    };
    msg.channel_id.say(&ctx.http, format!("After: {}", URLS.lock().unwrap()).as_str()).await?;
    Ok(())
}

fn save_raw(filename: &str, data: &[u8]) -> Result<(), IOError> {
    let mut file = File::create(filename)?;
    file.write_all(data)?;
    Ok(())
}

#[allow(dead_code)]
fn load_urls(filename: &str) -> Result<FolderSet, IOError> {
    let mut file = File::open(filename)?;
    let mut data = String::new();
    if let Err(e) = file.read_to_string(&mut data) {
        return Err(e);
    }
    
    match FolderSet::from_str(&mut data) {
        Ok(fs) => Ok(fs),
        Err(s) => Err(IOError::new(ErrorKind::Other, s))
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

#[test]
fn test_load() {
    let fs = match load_urls("data/urls.txt") {
        Ok(f) => f,
        Err(_) => nul(),
    };
    println!("'{}'", fs);
}

#[allow(dead_code)]
fn nul() -> FolderSet {
    println!("nul.");
    FolderSet::new()
}

#[allow(dead_code)]
fn caca(f: FolderSet) {
    println!("caca-> \n{} ", f);
}

