# Zip API

Compression helpers are available under `ptool.zip` and `p.zip`.

`ptool.zip` works on raw Lua strings, so it can be used for both text and
binary payloads.

Supported format names:

- `gzip` and `gz`
- `zlib`
- `deflate`
- `bzip2` and `bz2`
- `xz`
- `zstd`, `zst`, and `zstandard`

## ptool.zip.compress

> `v0.8.0` - Introduced.

`ptool.zip.compress(format, input)` compresses a Lua string with the requested
format.

- `format` (string, required): The compression format name.
- `input` (string, required): The input Lua string. Compression uses the
  string's raw bytes unchanged.
- Returns: `string` (compressed bytes as a Lua string).

Error behavior:

- An error is raised if `format` is not a supported format name.
- An error is raised if `input` is not a string.
- An error is raised if the encoder fails for the requested format.

Example:

```lua
local payload = p.fs.read("report.txt")
local compressed = p.zip.compress("gzip", payload)

p.fs.write("report.txt.gz", compressed)
```

## ptool.zip.decompress

> `v0.8.0` - Introduced.

`ptool.zip.decompress(format, input)` decompresses a Lua string with the
requested format.

- `format` (string, required): The compression format name.
- `input` (string, required): The compressed Lua string.
- Returns: `string` (decompressed bytes as a Lua string).

Error behavior:

- An error is raised if `format` is not a supported format name.
- An error is raised if `input` is not a string.
- An error is raised if `input` is not valid data for the requested format.

Example:

```lua
local compressed = p.fs.read("report.txt.gz")
local plain = p.zip.decompress("gzip", compressed)

print(plain)
```

Notes:

- `ptool.zip` does not infer formats from file names. Pass the format
  explicitly.
- `ptool.zip` operates on a single byte string and does not expose ZIP
  archive-entry APIs such as listing files inside a `.zip` container.
