fn main() {
    let exit_code = zstar_app_host::run(std::env::args());
    std::process::exit(exit_code);
}
