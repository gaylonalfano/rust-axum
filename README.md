```sh
# Terminal 1 - To run the server
cargo watch -qcw src/ -w .cargo/ -x "run"

# Terminal 2 - To run the quick_dev
cargo watch -qcw examples/ -x "run --example quick_dev"
```
