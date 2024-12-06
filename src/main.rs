use std::process::{Command, Stdio, Child};
use std::io::Read;
use std::thread;
use std::time::Duration;
use std::fs::File;
use std::process::ChildStdout;

fn update_discord_installation(installation_path: &str) {
    println!("Download new version...");
    let download_url = "https://discord.com/api/download/stable?platform=linux&format=tar.gz"; 
    let filename = "discord_setup.tar.gz";    

    let mut resp = reqwest::blocking::get(download_url).expect("request failed");
    let mut out = File::create(filename).expect("failed to create file");
    let n = resp.copy_to(&mut out).expect("Write to file failed.");
    println!("New version downloaded. {} bytes written to file.", n);
  
    println!("Unpacking file {}", filename); 
    let command = "tar";
    let args = ["-xzf", filename, "-C", "extract/"];
    let _tar = Command::new(command)
        .args(&args)
        .spawn()
        .expect("Could not unpack file.");
    println!("File unpacked");

    println!("Deleting old version found at {}", installation_path);
    let command = "rm";
    let args = ["-r", installation_path];
    let _rm = Command::new(command)
        .args(&args)
        .spawn();
    match _rm {
        Ok(_) => (),
        Err(e) => println!("{:?}", e.raw_os_error()),
    };
    println!("Old version deleted.");

    println!("Moving new version to destination {}", installation_path);
    let command = "mv";
    let source = "extract/Discord";
    let (destination,_) = installation_path.split_at(installation_path.len() - 1);
    let args = [&source, destination];
    println!("Moving downloaded version to {}", destination);
    let _mv = Command::new(command)
        .args(&args)
        .spawn()
        .expect("Could not move new version to PATH destination.");
    println!("New version moved to PATH destination.")


}

fn check_for_update(mut handle: ChildStdout) -> (bool,ChildStdout) {
    // Give discord time to check for updates
    thread::sleep(Duration::from_millis(2000));
    const BUF_SIZE: usize = 4096;
    let mut buf = [0;BUF_SIZE];
    let mut read_wrap = handle.take(BUF_SIZE.try_into().unwrap());
    let _ = read_wrap.read(&mut buf);
    handle = read_wrap.into_inner();

    let discord_output: &str = match std::str::from_utf8(&buf) {
        Ok(text) => text,
        Err(_) => panic!("Invalid utf-8 text received from discord."),
    };
    println!("{}", discord_output);
    let update_manually = "update-manually";

    for line in discord_output.lines() {
        if line.contains(update_manually) {
            println!("Discord requires manual update.");
            return (true,handle); 
        }
    }

    (false,handle)
}

fn run_discord(path: &str) -> Result<Child,std::io::Error> {
    let command = format!("{}{}", path, "Discord");
    Command::new(&command)
        .stdout(Stdio::piped())
        .spawn()
}

fn main() {
    let discord_path = "/home/lars/programs/Discord/";
    let p: Result<Child,std::io::Error> = run_discord(discord_path);

    let mut handle = match p {
        Ok(mut p) => {
            let handle = p.stdout.take().unwrap();
            let (needs_update,mut handle) = check_for_update(handle);
            if needs_update {
                let _ = p.kill();
                update_discord_installation(discord_path);
                p = run_discord(discord_path).unwrap();
                handle = p.stdout.take().unwrap();
            }
            handle
        }, 
        Err(e) => {
            let handle = match e.kind() {
                std::io::ErrorKind::NotFound =>  {
                    update_discord_installation(discord_path);
                    let mut p = run_discord(discord_path).unwrap();
                    let handle = p.stdout.take().unwrap();
                    handle
                },
                error => panic!("Got error when trying to open discord:\n{}",error),
            };
            handle
        },
    };

    let mut buf = String::new();
    let _ = handle.read_to_string(&mut buf);
}
