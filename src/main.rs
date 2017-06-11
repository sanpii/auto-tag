extern crate id3;
extern crate docopt;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate walkdir;
extern crate regex;
extern crate ansi_term;

use id3::Tag;
use std::ffi::OsStr;
use docopt::Docopt;
use walkdir::WalkDir;
use regex::Regex;
use ansi_term::Colour;

static USAGE: &'static str = "Usage: auto-tag [--dry-run] <path>";

#[derive(Deserialize)]
struct Args
{
    flag_dry_run: bool,
    arg_path: String,
}

fn main()
{
    let docopt = match Docopt::new(USAGE) {
        Ok(d) => d,
        Err(e) => e.exit(),
    };

    let args: Args = match docopt.deserialize() {
        Ok(args) => args,
        Err(e) => e.exit(),
    };

    for entry in WalkDir::new(args.arg_path) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let extension = match entry.path().extension() {
            Some(e) => e,
            None => continue,
        };

        if extension == OsStr::new("mp3") {
            print!("{:?}: ", entry.path());

            let mut tag = match get_tag(entry.path()) {
                Ok(tag) => tag,
                Err(e) => {
                    println!("{} ({})", Colour::Red.paint("failed"), e);
                    continue;
                },
            };

            if !args.flag_dry_run {
                match tag.write_to_path(entry.path()) {
                    Ok(_) => (),
                    Err(e) => {
                        println!("{} ({})", Colour::Red.paint("failed"), e);
                        continue;
                    },
                };
            }

            println!("{}", Colour::Green.paint("ok"));
        }
    }
}

fn get_tag(path: &std::path::Path) -> Result<Tag, String>
{
    let mut tag = match Tag::read_from_path(path) {
        Ok(tag) => tag,
        Err(_) => Tag::new(),
    };

    let regex = Regex::new(r"(?P<artist>[^/]+)/(?P<album>[^/]+)/(?P<track>\d+) - (?P<title>.+).mp3$")
        .unwrap();

    let captures = match regex.captures(path.to_str().unwrap()) {
        Some(c) => c,
        None => return Err(String::from("incompatible path")),
    };

    match captures.name("artist") {
        Some(artist) => tag.set_artist(artist),
        None => return Err(String::from("no artist info")),
    };

    match captures.name("album") {
        Some(album) => tag.set_album(album),
        None => return Err(String::from("no album info")),
    };

    match captures.name("title") {
        Some(title) => tag.set_title(title),
        None => return Err(String::from("no title info")),
    };

    match captures.name("track") {
        Some(track) => tag.set_track(track.parse().unwrap()),
        None => return Err(String::from("no track info")),
    };

    Ok(tag)
}
