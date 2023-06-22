#!/bin/bash

rm -rf ./tests/packages

# Generate and compare demo contracts
echo "Generating demo contracts..."
cargo build --bin gutenberg --no-default-features --features="cli"

for file in ./tests/scenarios/*.json; do
    echo "Generating $file"

    filename=$(basename -- "$file")
    filename="${filename%.*}"

    rm -rf ./tests/packages/demo/$filename/*
    ../../target/debug/gutenberg $file ./tests/packages/demo

    if [[ ! $? -eq 0 ]]; then
        echo "Scenario $file did not generate valid contract"
        echo "FAIL"
        exit 1
    fi
done

# Generate and compare full contracts
echo "Generating full contracts..."
cargo build --bin gutenberg --no-default-features --features="cli full"

for file in ./tests/scenarios/*.json; do
    echo "Generating $file"

    filename=$(basename -- "$file")
    filename="${filename%.*}"

    rm -rf ./tests/packages/full/$filename/*
    ../../target/debug/gutenberg $file ./tests/packages/full

    if [[ ! $? -eq 0 ]]; then
        echo "Scenario $file did not generate valid full contract"
        echo "FAIL"
        exit 1
    fi
done

echo "SUCCESS"