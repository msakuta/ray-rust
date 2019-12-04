while inotifywait -e close_write config.yaml; do cargo run --release -- -o foo.png -d config.yaml; done
