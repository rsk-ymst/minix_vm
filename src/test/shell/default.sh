#!/bin/bash
# テストコード実行用シェルスクリプト

asm_out_path=./out/default/

cargo build --release
./target/release/minix_vm ./bin/$1 > $asm_out_path/$1.txt