use std::fs::read;
use libavif_image;
use std::{fs};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::env;
use std::path::PathBuf;
fn avif_to_png(input_path: &str, output_path: &PathBuf){
    // Читаем AVIF-файл с помощью image
    let img = read(input_path);
    let binding = img.expect("REASON");
    let img: &[u8] = binding.as_slice();
    let img=libavif_image::read(img);
    let _=img.expect("REASON").save(output_path);

}
fn main() {
    let args: Vec<String> = env::args().collect();
    let input_directory;
    let output_directory;
    if args.len() != 5 {
        input_directory = "INPUT";
        output_directory = "OUTPUT"
    } else {
        input_directory = &args[2];
        output_directory = &args[4];
    }
    if !fs::metadata(output_directory).is_ok() {
        match fs::create_dir(output_directory) {
            Ok(_) => println!("Папка успешно создана: {}", output_directory),
            Err(e) => println!("Ошибка при создании папки: {}", e),
        }
    } else {
        println!("Папка уже существует: {}", output_directory);
    }

    if let Ok(paths) = fs::read_dir(input_directory) {
        // Клонирование ReadDir для избежания перемещения
        let paths = paths.collect::<Vec<_>>();

        // Получение общего количества файлов для прогресс-бара
        let total_files = paths.len() as u64;

        // Создание прогресс-бара
        let progress_bar = ProgressBar::new(total_files);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({eta})").expect("REASON")
                .progress_chars("##-"),
        );

        paths.par_iter().for_each(|path| {
            if let Ok(entry) = path {
                let path2 = entry.path();
                if path2.is_file() && path2.extension().is_some() {
                    // Получаем имя файла без расширения
                    let file_stem = path2.file_stem().expect("Failed to get file stem").to_string_lossy();

                    // Формируем новый путь с расширением PNG
                    let mut output_path = PathBuf::from(output_directory);
                    output_path.push(file_stem.to_string());
                    output_path.set_extension("png");

                    // Вызываем функцию avif_to_png с новым путем вывода
                    avif_to_png(path2.to_str().expect("Failed to convert path to string"), &output_path);

                    // Обновление прогресс-бара
                    progress_bar.inc(1);
                }
            }
        });
        progress_bar.finish();
    }
}