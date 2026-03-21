#!/usr/bin/env ptool run

p.use("v0.1.0")
p.config({ run = { check = true } })

-- Parse arguments.
local args = p.args.parse({
  name = "release.lua",
  about = "Help you release the project.",
  args = {
    p.args.arg("method", "positional"):required()
        :help("Release method, one of: alpha, beta, rc, patch, minor, major."),
  },
})

-- Validate arguments.
local valid_methods_re = "^(alpha|beta|rc|patch|minor|major)$"
if not p.re.compile(valid_methods_re):is_match(args.method) then
  local msg = (
    "Unknown release method `%s`, expected one of: alpha, beta, rc, patch, "
    .. "minor, major."
  ):format(args.method)
  error(msg, 0)
end

-- Ensure the working tree is clean before releasing. Only CHANGELOG.md is
-- allowed to have uncommitted changes (since the user updates it before
-- running this script). Without this check, untracked or uncommitted files
-- (such as .github/workflows/release.yml) would be missing from the release
-- commit and the tag, which could prevent CI workflows from triggering.
local status = p.run({
  cmd = "git",
  args = {"status", "--porcelain"},
  stdout = "capture",
})
local unexpected = {}
for line in status.stdout:gmatch("[^\n]+") do
  local file = line:match("^.. (.+)$")
  if file ~= "CHANGELOG.md" then
    unexpected[#unexpected + 1] = line
  end
end
if #unexpected > 0 then
  error(
    "Working tree is not clean. Please commit all changes before releasing:\n"
    .. table.concat(unexpected, "\n"), 0
  )
end

-- Update the version in Cargo.toml.
local cargo_text = p.fs.read("Cargo.toml")
local version_path = { "package", "version" }
local current_version = p.semver.parse(p.toml.get(cargo_text, version_path))
local next_version = current_version:bump(args.method)
local next_version_str = next_version:to_string()
local updated_cargo = p.toml.set(
  cargo_text, version_path, next_version_str
)
p.fs.write("Cargo.toml", updated_cargo)

-- Commit, tag, and push.
local tag_name = "v" .. next_version_str
p.run("cargo build")
p.run("echo", {"I am sure that I updated CHANGELOG.md"}, { confirm = true })
p.run("git add Cargo.toml Cargo.lock CHANGELOG.md")
p.run(
  "git",
  {"commit", "-m", 'chore: Release "' .. next_version_str .. '"'},
  { confirm = true }
)
p.run("git tag " .. tag_name)
p.run("git push origin HEAD", { confirm = true })
p.run("git push origin " .. tag_name, { confirm = true })

-- Show related information.
print(("Release method: %s"):format(args.method))
print(("Version: %s -> %s"):format(current_version, next_version))
print(("Tag: %s"):format(tag_name))
