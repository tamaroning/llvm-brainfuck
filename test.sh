#!/bin/bash
DEBUG="$(cd $(dirname $0); pwd)/target/debug/"

assert() {
    expected="$1"
    input="$2"

    actual=`${DEBUG}/llvm-brainfuck "$2"`
    
    if [ "$actual" = "$expected" ]; then
        echo -n '.'
    else
        echo -e "\n'$expected' is expected, but got '$actual'"
        exit 1
    fi
}

cargo build
assert % '+++++++++++++++++++++++++----++++----++++++++++++---+++++++.+-+-'
assert ABC '+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++.+.+.>++++++++++.'

echo OK
