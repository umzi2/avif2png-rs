use std::fs::{self, read_dir};
use std::path::{Path, PathBuf};
use std::env;
use indicatif::{ProgressBar, ProgressStyle};
use avif_decode::*;
use rgb::ComponentMap;
use std::sync::{Arc, Mutex};
use rayon::prelude::*;

fn avif_to_png(input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read(input_path).map_err(|e| format!("Unable to read '{}', because: {}", input_path.display(), e))?;
    let d = Decoder::from_avif(&data)?;
    let encoded = match d.to_image()? {
        Image::Rgb8(img) => {
            let (buf, width, height) = img.into_contiguous_buf();
            lodepng::encode_memory(&buf, width, height, lodepng::ColorType::RGB, 8)?
        }
        Image::Rgb16(img) => {
            let (mut buf, width, height) = img.into_contiguous_buf();
            buf.iter_mut().for_each(|px| {
                *px = px.map(|c| u16::from_ne_bytes(c.to_be_bytes()));
            });
            lodepng::encode_memory(&buf, width, height, lodepng::ColorType::RGB, 16)?
        }
        Image::Rgba8(img) => {
            let (buf, width, height) = img.into_contiguous_buf();
            lodepng::encode_memory(&buf, width, height, lodepng::ColorType::RGBA, 8)?
        }
        Image::Rgba16(img) => {
            let (mut buf, width, height) = img.into_contiguous_buf();
            buf.iter_mut().for_each(|px| {
                *px = px.map(|c| u16::from_ne_bytes(c.to_be_bytes()));
            });
            lodepng::encode_memory(&buf, width, height, lodepng::ColorType::RGBA, 16)?
        }
        Image::Gray8(img) => {
            let (buf, width, height) = img.into_contiguous_buf();
            lodepng::encode_memory(&buf, width, height, lodepng::ColorType::GREY, 8)?
        }
        Image::Gray16(img) => {
            let (mut buf, width, height) = img.into_contiguous_buf();
            buf.iter_mut().for_each(|px| {
                *px = px.map(|c| u16::from_ne_bytes(c.to_be_bytes()));
            });
            lodepng::encode_memory(&buf, width, height, lodepng::ColorType::GREY, 16)?
        }
    };

    fs::write(output_path, encoded).map_err(|e| format!("Unable to write '{}', because: {}", output_path.display(), e))?;
    Ok(())
}


fn process_directory(input_directory: &Path, output_directory: &Path, recursive: bool) -> Vec<(PathBuf, PathBuf)> {
    let mut files_to_process = Vec::new();

    if let Ok(paths) = read_dir(input_directory) {
        for path in paths.flatten() {
            let path2 = path.path();

            if path2.is_file() && path2.extension().is_some() {
                let file_stem = path2.file_stem().expect("Failed to get file stem").to_string_lossy();
                let mut output_path = PathBuf::from(output_directory);
                output_path.push(file_stem.to_string());
                output_path.set_extension("png");

                files_to_process.push((path2.clone(), output_path));
            } else if recursive && path2.is_dir() {
                let relative_path = path2.strip_prefix(input_directory).expect("Failed to get relative path");
                let new_output_directory = output_directory.join(relative_path);

                // Проверяем, существует ли каталог в выходной директории, и создаем его, если нет
                if !new_output_directory.exists() {
                    fs::create_dir_all(&new_output_directory).expect("Failed to create output directory");
                }

                let subdirectory_files = process_directory(&path2, &new_output_directory, recursive);
                files_to_process.extend(subdirectory_files);
            }
        }
    }

    files_to_process
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1); // First argument is program name
    let mut input_directory = String::from("INPUT");
    let mut output_directory = String::from("OUTPUT");
    let mut recursive = true;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-i" | "--input" => {
                if let Some(dir) = args.next() {
                    input_directory = dir.clone();
                } else {
                    eprintln!("Missing value for input directory");
                    return Ok(());
                }
            }
            "-o" | "--output" => {
                if let Some(dir) = args.next() {
                    output_directory = dir.clone();
                } else {
                    eprintln!("Missing value for output directory");
                    return Ok(());
                }
            }
            "--recursive" => recursive = true,
            _ => {
                eprintln!("Unknown argument: {}", arg);
                return Ok(());
            }
        }
    }

    let input_directory: &str = input_directory.as_str();
    let output_directory: &str = output_directory.as_str();
    if !fs::metadata(output_directory).is_ok() {
        fs::create_dir(output_directory)?;

    }

    let files_to_process = process_directory(Path::new(input_directory), Path::new(output_directory), recursive);
    let total_files = files_to_process.len() as u64;

    let progress_bar = Arc::new(Mutex::new(ProgressBar::new(total_files)));
    progress_bar.lock().unwrap().set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({eta})")?
            .progress_chars("##-"),
    );

    files_to_process.par_iter().for_each(|(input_path, output_path)| {
        if let Err(e) = avif_to_png(&input_path, &output_path) {
            eprintln!("Ошибка при обработке файла {}: {}", input_path.display(), e);
        }

        progress_bar.lock().unwrap().inc(1);
    });

    progress_bar.lock().unwrap().finish();

    Ok(())
}