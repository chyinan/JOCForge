#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
    echo "usage: ffmpeg-configure.sh SOURCE_DIR PREFIX" >&2
    exit 2
fi

source_dir=$1
prefix=$2
canonical_prefix=/jocforge-ffmpeg-prefix
cd "$source_dir"

./configure \
    --prefix="$canonical_prefix" \
    --target-os=mingw32 \
    --arch=x86_64 \
    --enable-static \
    --disable-shared \
    --disable-everything \
    --disable-autodetect \
    --disable-network \
    --disable-ffplay \
    --disable-doc \
    --disable-x86asm \
    --extra-ldflags=-static \
    --enable-encoder=ac3 \
    --enable-encoder=eac3 \
    --enable-encoder=pcm_s16le \
    --enable-decoder=ac3 \
    --enable-decoder=eac3 \
    --enable-decoder=pcm_f32le \
    --enable-demuxer=ac3 \
    --enable-demuxer=eac3 \
    --enable-demuxer=pcm_f32le \
    --enable-demuxer=mov \
    --enable-demuxer=wav \
    --enable-muxer=ac3 \
    --enable-muxer=eac3 \
    --enable-muxer=null \
    --enable-parser=ac3 \
    --enable-protocol=file \
    --enable-protocol=pipe \
    --enable-filter=anull \
    --enable-filter=aresample

make -j"${NUMBER_OF_PROCESSORS:-2}"
mkdir -p "$prefix/bin"
cp -f ffmpeg.exe ffprobe.exe "$prefix/bin/"
