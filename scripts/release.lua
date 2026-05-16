#!/usr/bin/env ptool

p.use("v0.6.0")
p.config { run = { check = true } }

-- Parse arguments.
local args = p.args.parse({
  name = "release.lua",
  about = "Help you release the project.",
  args = {
    p.args.arg("method", "positional"):required()
        :help(
          "Release method, one of: alpha, beta, rc, release, patch, minor, "
          .. "major, prepatch, preminor, premajor."
        ),
    p.args.arg("channel", "string", {
      long = "channel",
      help = "Prerelease channel for prepatch/preminor/premajor: alpha, beta, or rc.",
    }),
  },
})

-- Validate arguments.
local valid_methods_re = "^(alpha|beta|rc|release|patch|minor|major|prepatch|preminor|premajor)$"
if not p.re.compile(valid_methods_re):is_match(args.method) then
  local msg = (
    "Unknown release method `%s`, expected one of: alpha, beta, rc, release, "
    .. "patch, minor, major, prepatch, preminor, premajor."
  ):format(args.method)
  error(msg, 0)
end

local valid_channels_re = "^(alpha|beta|rc)$"
local is_pre_method = p.re.compile("^pre(patch|minor|major)$"):is_match(args.method)
if args.channel ~= nil and not p.re.compile(valid_channels_re):is_match(args.channel) then
  error(
    ("Unknown prerelease channel `%s`, expected one of: alpha, beta, rc."):format(args.channel),
    0
  )
end
if args.channel ~= nil and not is_pre_method then
  error("`--channel` is only supported with prepatch, preminor, or premajor.", 0)
end

-- Update the version in Cargo.toml.
local cargo_text = p.fs.read("Cargo.toml")
local version_path = { "workspace", "package", "version" }
local current_version = p.semver.parse(p.toml.get(cargo_text, version_path))
local next_version = current_version:bump(args.method, args.channel)
local next_version_str = next_version:to_string()
local updated_cargo = p.toml.set(
  cargo_text, version_path, next_version_str
)
p.fs.write("Cargo.toml", updated_cargo)

-- If the version is a stable release, update the website documentation.
local is_stable_release = next_version.pre == nil
if is_stable_release then
  p.run(
    "node website/scripts/manage-doc-versions.mjs snapshot "
      .. next_version_str
      .. " --keep 7"
  )
  p.run("cd website && npm run i18n:refresh-docs")
end

-- Commit, tag, and push.
local tag_name = "v" .. next_version_str
p.run("cargo build")
local update_changelog = p.ask.confirm("Have you updated CHANGELOG.md?", {
  default = false,
})
if not update_changelog then
  error("Please update the CHANGELOG.md file before releasing.", 0)
end
if is_stable_release then
  p.run(
    "git add CHANGELOG.md Cargo.toml Cargo.lock"
      .. " website/versions.json"
      .. " website/versioned_docs"
      .. " website/versioned_sidebars"
      .. " website/po/docs"
      .. " website/i18n"
  )
else
  p.run("git add CHANGELOG.md Cargo.toml Cargo.lock")
end
p.run(
  "git",
  { "commit", "-m", 'chore: Release "' .. next_version_str .. '".' },
  { confirm = true }
)
p.run("git tag " .. tag_name)
p.run("git push origin HEAD", { confirm = true })
p.run("git push origin " .. tag_name, { confirm = true })

-- Show related information.
p.log.info(("Release method: %s"):format(args.method))
if is_pre_method then
  p.log.info(("Prerelease channel: %s"):format(args.channel or "alpha"))
end
p.log.info(("Version: %s -> %s"):format(current_version, next_version))
p.log.info(("Tag: %s"):format(tag_name))
