extern crate winres;
use std::env;

fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        let exe = env::current_exe().unwrap();
        let contents = exe
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_str()
            .unwrap();
        res.set_icon(&format!("{}/assets/icon.ico", contents));
        res.compile().unwrap();
    }
}
