use std::{error, fs, io, path::Path, sync::atomic::{AtomicBool, Ordering}, thread, time::Duration};
use rand::{seq::SliceRandom, thread_rng, Rng};
use rodio::{Decoder, OutputStream, Sink};

fn main() {
    let sound_directory = "./src/sounds";

    let sound_files: Vec<_> = fs::read_dir(sound_directory)
        .expect("Failed to read directory")
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "mp3") {
                Some(path.to_str().unwrap().to_owned())
            } else {
                None
            }
        })
        .collect();

    if sound_files.is_empty() {
        println!("No mp3 files found in the directory");
        return;
    }

    println!("Welcome to Echo of the Wild.");
    println!("Enter a command: ");

    let mut echo_thread_handle: Option<thread::JoinHandle<()>> = None;
    // let stop_flag = Arc::new(AtomicBool::new(false));
    let input_thread_handle = thread::spawn(move || {

        let mut input = String::new();
        let mut is_echo_running = false;

        loop {   
            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    println!("Received: {}", input);
                    match input.trim() {
                        "init" => {
                            if is_echo_running {
                                println!("Echo of the Wild is already running.")
                            } else {
                                is_echo_running = true;
                                let files_clone = sound_files.clone();
                                echo_thread_handle = Some(thread::spawn(move || {
                                    if let Err(error) = echo_of_wild(&files_clone) {
                                        eprintln!("Error on echo_of_wild run: {}", error);
                                    }
                                }));
                            }
                        },
                        "stop" => break,
                        _ => println!("Invalid command"),
                    }
                }
                Err(error) => {
                    eprintln!("Error: {}", error);
                    break;
                }
            }
            input.clear();
        }

        if let Some(handle) = echo_thread_handle {
            handle.join().expect("Echo thread paniked");
        }
    });

    input_thread_handle.join().expect("Input thread panicked");
}

fn echo_of_wild(files: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;

    loop {
        let sound_path = files.choose(&mut thread_rng()).unwrap();

        let sound = fs::File::open(sound_path)?;
        let decoder = Decoder::new(sound)?;

        sink.append(decoder);

        sink.play();

        while sink.len() > 0 {
            thread::sleep(Duration::from_millis(100));
        }

        // wait before next sound
        let interval = Duration::from_secs(thread_rng().gen_range(1..11));
        println!("Waiting for {} seconds", interval.as_secs());
        thread::sleep(interval);
    }
}
