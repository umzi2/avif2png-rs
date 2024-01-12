build:
```
git clone https://github.com/umzi2/avif2png-rs
cargo build --release
```

linux:
```
chmod +x avif2png-rs 
./avif2png-rs -i <путь> -o <путь> --recursive(опционально)
```
windows:
```
avif2png-rs.exe -i <путь> -o <путь> --recursive(опционально)
```
За основу был взять репозиторий kornelski: [avif-decode](https://github.com/kornelski/avif-decode)
