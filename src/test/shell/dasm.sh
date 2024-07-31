#!/bin/bash
# テストコード実行用シェルスクリプト
# ディスアセンブルモード用

asm_out_path=./out/asm/

cargo run -- -d ./bin/$1 > $asm_out_path/$1.txt