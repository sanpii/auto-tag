#![warn(warnings)]

use clap::Parser;
use id3::TagLike;
use nu_ansi_term::Color;

type Result<T = ()> = anyhow::Result<T>;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Incompatible path")]
    IncompatiblePath,
}

#[derive(Parser)]
struct Opt {
    #[arg(long)]
    dry_run: bool,
    #[arg(default_value_t = String::from("."))]
    path: String,
}

fn main() {
    let opt = Opt::parse();

    for entry in walkdir::WalkDir::new(opt.path) {
        let Ok(entry) = entry else {
            continue;
        };

        let Ok(path) = entry.path().canonicalize() else {
            continue;
        };

        let Some(extension) = path.extension() else {
            continue;
        };

        if extension == "mp3" {
            print!("'{}': ", path.display());

            let tag = match tag(&path) {
                Ok(tag) => tag,
                Err(e) => {
                    println!("{} ({e})", Color::Red.paint("failed"));
                    continue;
                }
            };

            if !opt.dry_run
                && let Err(err) = tag.write_to_path(&path, id3::Version::Id3v24)
            {
                println!("{} ({err})", Color::Red.paint("failed"));
                continue;
            }

            println!("{}", Color::Green.paint("ok"));
        }
    }
}

fn tag(path: &std::path::Path) -> Result<id3::Tag> {
    let mut tag = match id3::Tag::read_from_path(path) {
        Ok(tag) => tag,
        Err(_) => id3::Tag::new(),
    };

    ctreg::regex! { Regex = r"(?P<artist>[^/]+)/(?P<album>[^/]+)/(?P<track>\d+) - (?P<title>.+).mp3$" };

    let regex = Regex::new();

    let Some(captures) = regex.captures(path.to_str().unwrap_or_default()) else {
        return Err(Error::IncompatiblePath.into());
    };

    tag.set_artist(captures.artist.content);
    tag.set_album_artist(captures.artist.content);
    tag.set_album(captures.album.content);
    tag.set_title(captures.title.content);
    tag.set_track(captures.track.content.parse()?);

    Ok(tag)
}
