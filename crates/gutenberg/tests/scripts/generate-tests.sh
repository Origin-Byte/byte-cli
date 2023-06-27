#!/bin/bash

cargo build --bin gutenberg --features="cli"
if [[ ! $? -eq 0 ]]; then
    echo "Could not build Gutenberg"
    echo "FAIL"
    exit 1
fi

rm -rf ./tests/packages

# Generate and compare demo contracts
echo "Generating demo contracts..."

for file in ./tests/scenarios/*.json; do
    echo "Generating $file"

    filename=$(basename -- "$file")
    filename="${filename%.*}"

    rm -rf ./tests/packages/demo/$filename/*
    ../../target/debug/gutenberg --demo $file ./tests/packages/demo

    if [[ ! $? -eq 0 ]]; then
        echo "Scenario $file did not generate valid contract"
        echo "FAIL"
        exit 1
    fi
done

# Generate and compare full contracts
echo "Generating full contracts..."

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