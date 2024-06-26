use std::{error, fs, io, path::Path, sync::{Arc, atomic::{AtomicBool, Ordering}}, thread, time::Duration};
use rand::{seq::SliceRandom, thread_rng, Rng};
use rodio::{Decoder, OutputStream, Sink};

mod adjust_sound;

use adjust_sound::{get_volume_level, adjust_volume};

fn main() {
    let sound_directory = "./sounds";
    let output_directory = "./src/ready_sounds";

    let sound_files: Vec<_> = fs::read_dir(sound_directory)
        .expect("Failed to read directory")
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "wav") {
                Some(path.to_str().unwrap().to_owned())
            } else {
                None
            }
        })
        .collect();

    let ready_sound_files: Vec<_> = fs::read_dir(output_directory)
        .expect("Failed to read directory")
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "wav") {
                Some(path.to_str().unwrap().to_owned())
            } else {
                None
            }
        })
        .collect();

    if sound_files.is_empty() {
        println!("No wav files found in the directory");
        return;
    }

    // The outputed files are silent for some reason
    // if let Err(error) = adjust_sounds(&sound_files, output_directory) {
    //     eprintln!("Error adjusting sounds: {}", error);
    // }

    println!("Welcome to Echo of the Wild.");
    println!("Enter a command: ");

    let stop_flag = Arc::new(AtomicBool::new(false));
    let input_thread_handle = thread::spawn({
            let stop_flag = stop_flag.clone();
            move || {
            let mut is_echo_running = false;

            loop {   
                let mut input = String::new();
                match io::stdin().read_line(&mut input) {
                    Ok(_) => {
                        println!("Received: {}", input);
                        match input.trim() {
                            "init" => {
                                if is_echo_running {
                                    println!("Echo of the Wild is already running.")
                                } else {
                                    is_echo_running = true;
                                    stop_flag.store(false, Ordering::SeqCst);
                                    let files_clone = sound_files.clone();
                                    let stop_flag_clone = stop_flag.clone();
                                    thread::spawn(move || {
                                        if let Err(error) = echo_of_wild(&files_clone, &stop_flag_clone) {
                                            eprintln!("Error on echo_of_wild run: {}", error);
                                        }
                                    });
                                }
                            },
                            "stop" => {
                                stop_flag.store(true, Ordering::SeqCst);
                                is_echo_running = false;
                            },
                            "exit" => break,
                            _ => println!("Invalid command"),
                        }
                    }
                    Err(error) => {
                        eprintln!("Error: {}", error);
                        break;
                    }
                }
            }
        }
    });

    input_thread_handle.join().expect("Input thread panicked");
}

fn echo_of_wild(files: &[String], stop_flag: &Arc<AtomicBool>) -> Result<(), Box<dyn std::error::Error>> {
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;

    loop {
        if stop_flag.load(Ordering::SeqCst) {
            break;
        }

        let sound_path = files.choose(&mut thread_rng()).unwrap();

        let sound = fs::File::open(sound_path)?;
        let decoder = Decoder::new(sound)?;

        sink.append(decoder);

        sink.play();

        while sink.len() > 0 {
            thread::sleep(Duration::from_millis(100));
        }
        if stop_flag.load(Ordering::SeqCst) {
            break;
        }
        // wait before next sound
        let interval = Duration::from_secs(thread_rng().gen_range(1..180));
        println!("Waiting for {} seconds", interval.as_secs());
        thread::sleep(interval);
    }

    Ok(())
}

fn adjust_sounds(sound_files: &Vec<String>, output_directory: &str) -> Result<(), Box<dyn std::error::Error>> {
    for file in sound_files {
        let volume_level = get_volume_level(file)?;

        let volume_factor = 10.0 / volume_level;

        let file_name = Path::new(file).file_name().unwrap().to_str().unwrap();
        let output_file_path = Path::new(output_directory).join(file_name);
        adjust_volume(file, output_file_path.to_str().unwrap(), volume_factor)?;
    }

    Ok(())
}
