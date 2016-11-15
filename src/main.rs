extern crate id3;
extern crate docopt;
extern crate rustc_serialize;
extern crate walkdir;
extern crate regex;
extern crate ansi_term;

use id3::Tag;
use std::ffi::OsStr;
use docopt::Docopt;
use walkdir::WalkDir;
use regex::Regex;
use ansi_term::Colour;

static USAGE: &'static str = "Usage: tag <path>";

#[derive(RustcDecodable)]
struct Args
{
    arg_path: String,
}

fn main()
{
    let docopt = match Docopt::new(USAGE) {
        Ok(d) => d,
        Err(e) => e.exit(),
    };

    let args: Args = match docopt.decode() {
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

            match write_tags(entry.path()) {
                Ok(_) => println!("{}", Colour::Green.paint("ok")),
                Err(e) => println!("{} ({})", Colour::Red.paint("failed"), e),
            };
        }
    }
}

fn write_tags(path: &std::path::Path) -> Result<(), String>
{
    let mut tag = Tag::new();

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

    match tag.write_to_path(path) {
        Ok(_) => Ok(()),
        Err(e) => Err(String::from(e.description)),
    }
}
