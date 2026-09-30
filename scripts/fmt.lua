#!/usr/bin/env ptool

p.use("v0.11.0")
p.config { run = { check = true } }

p.run("cargo fmt --all")
