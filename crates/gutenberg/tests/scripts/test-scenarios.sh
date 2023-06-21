#!/bin/bash
tmp_dir=$(mktemp -d)

# Generate and compare full contracts
#
# Test full contracts first since they're the ones most likely to fail
echo "Testing generated full contracts..."
cargo build --bin gutenberg --no-default-features --features="cli full"

for file in ./tests/scenarios/*.json; do
    echo "Testing scenario $file"

    rm -rf $tmp_dir/*
    ../../target/debug/gutenberg $file $tmp_dir

    filename=$(basename -- "$file")
    filename="${filename%.*}"
    diff -r ./tests/packages/full/$filename $tmp_dir/$filename

    if [ $? -eq 1 ]; then
        echo "Scenario $file did not generate matching full contract"
        echo "Run './tests/generate-tests.sh' to update tests"
        echo "FAIL"
        rm -rf $tmp_dir
        exit 1
    fi

    # Run Sui tests
    #
    # Output is silenced unless an error occurs
    output=$(sui move test --path $tmp_dir/$filename 2>/dev/null)

    # `sui move test` always returns zero code
    #
    # If tests will fail, it will print to stderr
    # If they wont fail, it will print everything to stderr
    #
    # Cant grep on `sui move test` as this will break the pipe
    #
    # ¯\_(ツ)_/¯
    if [[ ! -z $(echo $output | grep -F "error") ]]; then
        echo "$output"
        echo "Scenario $file did not pass Sui tests"
        echo "FAIL"
        rm -rf $tmp_dir
        exit 1
    fi
done

# Generate and compare demo contracts
echo "Testing generated demo contracts..."
cargo build --bin gutenberg --no-default-features --features="cli"

for file in ./tests/scenarios/*.json; do
    echo "Testing scenario $file"

    rm -rf $tmp_dir/*
    ../../target/debug/gutenberg $file $tmp_dir

    filename=$(basename -- "$file")
    filename="${filename%.*}"
    diff -r ./tests/packages/demo/$filename $tmp_dir/$filename

    if [ $? -eq 1 ]; then
        echo "Scenario $file did not generate matching demo contract"
        echo "Run './tests/generate-tests.sh' to update tests"
        echo "FAIL"
        rm -rf $tmp_dir
        exit 1
    fi

    # Run Sui tests
    output=$(sui move test --path $tmp_dir/$filename 2>/dev/null)

    if [[ ! -z $(echo $output | grep -F "error") ]]; then
        echo "$output"
        echo "Scenario $file did not pass Sui tests"
        echo "FAIL"
        rm -rf $tmp_dir
        exit 1
    fi
done

rm -rf $tmp_dir

echo "SUCCESS"