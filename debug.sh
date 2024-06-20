#! /usr/bin/env bash

if [[ "$1" == "r" ]]
then
    cargo run -- -gdb tcp::4242 -S
else
    gdb -x build/debug.gdb
fi
