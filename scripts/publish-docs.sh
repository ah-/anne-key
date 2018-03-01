#!/bin/sh

set -euxo pipefail

pushd book

echo -e "Initializing Git"
git init
git config user.name "Andreas Heider"
git config user.email "andreas@heider.io"

git remote add upstream "https://$GH_TOKEN@github.com/ah-/anne-key.git"
git fetch upstream --quiet
git reset upstream/gh-pages --quiet

touch .

echo -e "Pushing changes to gh-pages"
git add -A . 
git commit -m "rebuild pages at ${rev}" --quiet
git push -q upstream HEAD:gh-pages

echo -e "Deployed docs to GitHub Pages"

popd
