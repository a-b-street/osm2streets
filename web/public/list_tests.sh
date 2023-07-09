#!/bin/sh

file=public/tests.json
ls public/tests/ | sed -e 's/\(.*\)/  \"\1\"/' > $file
# https://stackoverflow.com/a/35021663
sed -i '1s/^/[/;$!s/$/,/;$s/$/]/' $file
