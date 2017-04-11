#!/bin/bash

# This runs only when a commit is pushed to master. It is responsible for
# updating docs and computing coverage statistics.

set -e

#if [ "$TRAVIS_RUST_VERSION" != "stable" ] || [ "$TRAVIS_PULL_REQUEST" != "false" ] || [ "$TRAVIS_BRANCH" != "master" ]; then
if [ "$TRAVIS_RUST_VERSION" != "stable" ] || [ "$TRAVIS_PULL_REQUEST" != "false" ]; then
  exit 0
fi

env

# Build and upload docs.
cargo doc --verbose
echo '<meta http-equiv=refresh content=0;url=regex/index.html>' > target/doc/index.html
ve=$(mktemp -d)
virtualenv "$ve"
"$ve"/bin/pip install --upgrade pip
"$ve"/bin/pip install ghp-import
"$ve"/bin/ghp-import -n target/doc
#git push -qf https://${TOKEN}@github.com/${TRAVIS_REPO_SLUG}.git gh-pages

# Install kcov.
tmpdir=$(mktemp -d)
pushd "$tmpdir"
wget https://github.com/SimonKagstrom/kcov/archive/master.tar.gz
tar zxf master.tar.gz
mkdir kcov-master/build
cd kcov-master/build
cmake ..
make
make install DESTDIR="$tmpdir"
popd

PATH="$tmpdir/usr/local/bin:$PATH"

# Calculate and upload code coverage.
tmpdir=$(mktemp -d)
cargo test --no-run --verbose

find ./target/debug -maxdepth 1 -type f -executable | \
  xargs -n 1 basename | \
  xargs -n 1 -I CMD \
    kcov --exclude-pattern=/.cargo --verify "$tmpdir/CMD" ./target/debug/CMD

kcov --verify --coveralls-id=$TRAVIS_JOB_ID --merge target/cov "$tmpdir"/*
