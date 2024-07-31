#!/bin/bash
# テストコード実行用シェルスクリプト
# 5.c用

asm_out_path=./out/default/

cargo build --release
./target/release/minix_vm ./bin/5c hoge fuga piyo > $asm_out_path/5c.txt