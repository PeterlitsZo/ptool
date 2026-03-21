use std::fs as stdfs;
use std::path::Path;

pub(crate) fn read(path: String) -> mlua::Result<String> {
    stdfs::read_to_string(&path)
        .map_err(|err| mlua::Error::runtime(format!("ptool.fs.read `{path}` failed: {err}")))
}

pub(crate) fn write(path: String, content: String) -> mlua::Result<()> {
    stdfs::write(&path, content)
        .map_err(|err| mlua::Error::runtime(format!("ptool.fs.write `{path}` failed: {err}")))
}

pub(crate) fn mkdir(path: String) -> mlua::Result<()> {
    stdfs::create_dir_all(&path)
        .map_err(|err| mlua::Error::runtime(format!("ptool.fs.mkdir `{path}` failed: {err}")))
}

pub(crate) fn exists(path: String) -> bool {
    Path::new(&path).exists()
}
