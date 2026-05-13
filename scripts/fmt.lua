#!/usr/bin/env ptool

p.use("v0.5.0")
p.config { run = { check = true } }

p.run("cargo fmt --all")
