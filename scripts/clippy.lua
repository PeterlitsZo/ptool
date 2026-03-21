#!/usr/bin/env ptool run

p.use("v0.1.0")

p.run("cargo clippy --all-targets -- -D warnings")
