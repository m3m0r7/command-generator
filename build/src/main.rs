fn main() {
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string());
    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    println!("command-generator-build");
    println!("profile={profile}");
    println!("target={target}");
}
