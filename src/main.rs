use std::fs::{self, read_dir};
use std::path::{Path, PathBuf};
use std::env;
use indicatif::{ProgressBar, ProgressStyle};
use avif_decode::*;
use rgb::ComponentMap;
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let input_directory;
    let output_directory;

    if args.len() != 5 {
        input_directory = "INPUT";
        output_directory = "OUTPUT2";
    } else {
        input_directory = &args[2];
        output_directory = &args[4];
    }

    if !fs::metadata(output_directory).is_ok() {
        fs::create_dir(output_directory)?;
        println!("Папка успешно создана: {}", output_directory);
    } else {
        println!("Папка уже существует: {}", output_directory);
    }

    if let Ok(paths) = read_dir(input_directory) {
        let paths = paths.collect::<Vec<_>>();
        let total_files = paths.len() as u64;

        let progress_bar = ProgressBar::new(total_files);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({eta})")?
                .progress_chars("##-"),
        );

        paths.par_iter().for_each(|path| {
            if let Ok(entry) = path {
                let path2 = entry.path();
                if path2.is_file() && path2.extension().is_some() {
                    let file_stem = path2.file_stem().expect("Failed to get file stem").to_string_lossy();
                    let mut output_path = PathBuf::from(output_directory);
                    output_path.push(file_stem.to_string());
                    output_path.set_extension("png");

                    if let Err(e) = avif_to_png(&path2, &output_path) {
                        eprintln!("Ошибка при обработке файла {}: {}", path2.display(), e);
                    }

                    progress_bar.inc(1);
                }
            }
        });

        progress_bar.finish();
    }

    Ok(())
}
