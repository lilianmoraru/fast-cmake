use std::path::{ Path, PathBuf };
use std::collections::HashMap;
use std::sync::atomic::{ AtomicPtr, Ordering };
use std::boxed::Box;

lazy_static! {
    static ref BINARIES: AtomicPtr<HashMap<String, PathBuf>> =
        AtomicPtr::new(
            Box::into_raw(Box::new(HashMap::with_capacity(1024)))
        );
}

fn populate_binaries() {
    let os_path = ::std::env::var("PATH")
                             .and_then(|path| {
                                 if path.as_str().is_empty() {
                                     Err(::std::env::VarError::NotPresent)
                                 } else {
                                     Ok(path)
                                 }
                             });

    if let Err(_) = os_path { return; }

    let mut map = unsafe { Box::from_raw(BINARIES.load(Ordering::Relaxed)) };
    for path in os_path.unwrap().split(":") {
        let dir = Path::new(path);
        if dir.is_dir() {
            let read_dir = match ::std::fs::read_dir(dir) {
                Ok(result) => result,
                Err(_)     => continue,
            };

            for entry in read_dir {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(_)    => continue,
                };

                if let Ok(basename) = entry.file_name().into_string() {
                    map.entry(basename).or_insert(entry.path().to_path_buf());
                }
            }
        }
    }
    ::std::mem::forget(map);
}

pub fn find_program<S: Into<String>>(bin: S) -> Option<PathBuf> {
    use std::collections::hash_map::Entry;

    let binary: String = bin.into();
    let mut map = unsafe { Box::from_raw(BINARIES.load(Ordering::Relaxed)) };

    let path = match map.entry(binary.clone()) {
        Entry::Occupied(path) => Some(path.get().clone()),
        Entry::Vacant(_)      => {
            populate_binaries();
            None
        }
    };

    let path = path.or_else(|| {
        match map.entry(binary) {
            Entry::Occupied(path) => Some(path.get().clone()),
            Entry::Vacant(_)      => None,
        }
    });
    ::std::mem::forget(map);

    path
}

#[test]
fn test() {
    if cfg!(unix) {
        find_program("env").unwrap();
    } else if cfg!(windows) {
        find_program("notepad").unwrap();
    }
}
