cargo clean
cargo build

$env:RUSTFLAGS="-Cinstrument-coverage"
$env:LLVM_PROFILE_FILE="aetolia-%p-%m.profraw"

(Get-ChildItem -Path $Path).Fullname -match ".*.profraw" | Remove-Item
cargo test
grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o ./target/debug/coverage/
