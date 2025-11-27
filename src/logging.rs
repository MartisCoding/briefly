
pub fn init_logger() {
    use env_logger::Env;
    env_logger::Builder::from_env(
        Env::default().default_filter_or("info")
    ).init();
}

