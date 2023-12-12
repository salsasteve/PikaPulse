mod audio_clip;

use audio_clip::AudioClip;
use chrono::prelude::*;
use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;

#[derive(Parser, Debug)]
#[clap(name = "pikapulse", about = "CLI to record conversions for analysis", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Record an audio clip using the default input device until ctrl+c is pressed.
    Record {
        /// The name of the clip to record. If not specified, the current date and time will be used.
        clip_name: Option<String>,
        clip_length: Option<u64>,
    },
    /// List all clips.
    List,
    /// Play the clip with the given name.
    Play {
        /// The name of the clip to play.
        #[clap(required = true)]
        clip_name: String,
    },
    /// Delete the clip with the given name.
    Delete {
        /// The name of the clip to delete.
        #[clap(required = true)]
        clip_name: String,
    },
}

fn maind() -> Result<()> {
    color_eyre::install()?;
    let args = Cli::parse();
    // let mut audio_clips: HashMap<String, AudioClip> = HashMap::new();

    match args.command {
        Commands::Record {
            clip_name,
            clip_length,
        } => {
            let formatted_time = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
            let clip_name =
                clip_name.unwrap_or_else(|| format!("recording_{}.wav", formatted_time));

            let clip_length = clip_length.unwrap_or_else(|| 3);
            println!("Clip filename: {}", clip_name);
            println!("Clip length: {}", clip_length);

            let mut clip =
                AudioClip::new(clip_name.clone(), clip_length).expect("Failed to create AudioClip");

            clip.record().expect("Failed to record");

            // audio_clips.insert(clip_name, clip);
        }
        Commands::List => {
            // Implement list functionality here
            println!("List");
            // for (name, _clip) in audio_clips.iter() {
            //     println!("Clip name: {}", name);
            // }
        }
        Commands::Play { clip_name } => {
            println!("Play {}", clip_name);
            let _ = AudioClip::play(clip_name);
            // Implement play functionality here
        }
        Commands::Delete { clip_name } => {
            println!("Delete {}", clip_name);
            // Implement delete functionality here
        }
    }

    Ok(())
}
