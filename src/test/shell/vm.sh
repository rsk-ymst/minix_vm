#!/bin/bash
# テストコード実行用シェルスクリプト
# VMモード用

vm_out_path=./out/vm/

cargo run --features="vm"  -- -m ./bin/$1 > $vm_out_path/$1.txt