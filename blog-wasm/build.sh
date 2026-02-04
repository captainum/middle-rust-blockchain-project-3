#!/bin/bash

set -e

echo "Сборка WASM модуля..."
wasm-pack build --target web

echo "Перенос модуля в www..."
rm -rf www/pkg
mv pkg www/

echo "Сборка завершена!"
echo ""
echo "Запуск по команде: cd www && python3 -m http.server 8080"
