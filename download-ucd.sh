#!/bin/sh

set -ex

DIR="ucd"
VERSION="13.0.0"

download_and_unzip()
{
mkdir -p "$DIR"
curl -o "$DIR/$1.zip" "https://www.unicode.org/Public/$VERSION/ucd/$1.zip"
unzip "$DIR/$1.zip" -d "$DIR"
rm -f "$DIR/$1.zip"
}

rm -rf "$DIR"
download_and_unzip UCD
download_and_unzip Unihan
