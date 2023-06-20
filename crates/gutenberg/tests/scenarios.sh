#!/bin/bash
tmp_dir=$(mktemp -d)

# Generate and compare demo contracts
echo "Testing generated demo contracts..."
cargo build --bin gutenberg --features="cli"

for file in ./tests/scenarios/*.json; do
    echo "Testing scenario $file"

    rm -rf $tmp_dir/*
    ../../target/debug/gutenberg generate $file $tmp_dir

    filename=$(basename -- "$file")
    filename="${filename%.*}"
    diff -r ./tests/packages/demo/$filename $tmp_dir/$filename

    if [ $? -eq 1 ]; then
        echo "Scenario $file did not generate matching demo contract"
        echo "Run 'cargo run --bin gutenberg --features="cli" -- generate-tests' to update tests"
        echo "FAIL"
        rm -rf $tmp_dir
        exit 1
    fi

    # Run Sui tests
    sui move test --path $tmp_dir/$filename

    if [ $? -eq 1 ]; then
        echo "Scenario $file did not pass Sui tests"
        echo "FAIL"
        rm -rf $tmp_dir
        exit 1
    fi
done

# Generate and compare full contracts
echo "Testing generated full contracts..."
cargo build --bin gutenberg --features="cli full"

for file in ./tests/scenarios/*.json; do
    echo "Testing scenario $file"

    rm -rf $tmp_dir/*
    ../../target/debug/gutenberg generate $file $tmp_dir

    filename=$(basename -- "$file")
    filename="${filename%.*}"
    diff -r ./tests/packages/full/$filename $tmp_dir/$filename

    if [ $? -eq 1 ]; then
        echo "Scenario $file did not generate matching full contract"
        echo "Run 'cargo run --bin gutenberg --features="cli full" -- generate-tests' to update tests"
        echo "FAIL"
        rm -rf $tmp_dir
        exit 1
    fi

    # Run Sui tests
    sui move test --path $tmp_dir/$filename

    if [ $? -eq 1 ]; then
        echo "Scenario $file did not pass Sui tests"
        echo "FAIL"
        rm -rf $tmp_dir
        exit 1
    fi
done

rm -rf $tmp_dir


echo "SUCCESS"