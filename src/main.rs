#![warn(warnings)]

use id3::TagLike;
use std::ffi::OsStr;
use walkdir::WalkDir;
use regex::Regex;
use nu_ansi_term::Color;
use clap::Parser;

#[derive(Parser)]
struct Opt
{
    #[arg(long)]
    dry_run: bool,
    path: String,
}

fn main()
{
    let opt = Opt::parse();

    for entry in WalkDir::new(opt.path) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = match entry.path().canonicalize() {
            Ok(p) => p,
            Err(_) => continue,
        };

        let extension = match path.extension() {
            Some(e) => e,
            None => continue,
        };

        if extension == OsStr::new("mp3") {
            print!("{path:?}: ");

            let tag = match get_tag(&path) {
                Ok(tag) => tag,
                Err(e) => {
                    println!("{} ({})", Color::Red.paint("failed"), e);
                    continue;
                },
            };

            if !opt.dry_run {
                match tag.write_to_path(&path, id3::Version::Id3v24) {
                    Ok(_) => (),
                    Err(e) => {
                        println!("{} ({})", Color::Red.paint("failed"), e);
                        continue;
                    },
                };
            }

            println!("{}", Color::Green.paint("ok"));
        }
    }
}

fn get_tag(path: &std::path::Path) -> Result<id3::Tag, String>
{
    let mut tag = match id3::Tag::read_from_path(path) {
        Ok(tag) => tag,
        Err(_) => id3::Tag::new(),
    };

    let regex = Regex::new(r"(?P<artist>[^/]+)/(?P<album>[^/]+)/(?P<track>\d+) - (?P<title>.+).mp3$")
        .unwrap();

    let captures = match regex.captures(path.to_str().unwrap()) {
        Some(c) => c,
        None => return Err(String::from("incompatible path")),
    };

    match captures.name("artist") {
        Some(artist) => {
            tag.set_artist(artist.as_str());
            tag.set_album_artist(artist.as_str());
        },
        None => return Err(String::from("no artist info")),
    };

    match captures.name("album") {
        Some(album) => tag.set_album(album.as_str()),
        None => return Err(String::from("no album info")),
    };

    match captures.name("title") {
        Some(title) => tag.set_title(title.as_str()),
        None => return Err(String::from("no title info")),
    };

    match captures.name("track") {
        Some(track) => tag.set_track(track.as_str().parse().unwrap()),
        None => return Err(String::from("no track info")),
    };

    Ok(tag)
}
