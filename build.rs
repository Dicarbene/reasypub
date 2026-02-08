#[cfg(target_os = "windows")]
fn main() {
    let mut resource = winresource::WindowsResource::new();
    resource.set_icon("assets/favicon.ico");
    if let Err(error) = resource.compile() {
        panic!("failed to compile windows resources: {error}");
    }
}

#[cfg(not(target_os = "windows"))]
fn main() {}
