echo "\nrunning unit tests\n"
cd contract
cargo test
cd ..

echo "\nbuilding...\n"
./build_all.sh

echo "\nrunning integration tests\n"
cd integration-tests
cargo run --example integration-tests
cd ..