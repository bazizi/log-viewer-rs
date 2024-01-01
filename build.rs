#[cfg(target_os = "windows")]
fn main() {
    extern crate winres;
    let mut res = winres::WindowsResource::new();
    res.set_icon("res/log.ico"); // Replace this with the filename of your .ico file.
    res.compile().unwrap();
}

#[cfg(target_os = "linux")]
fn main(){
// no op
}
