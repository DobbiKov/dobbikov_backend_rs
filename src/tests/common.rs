use loggit::Level;

pub fn setup() {
    loggit::logger::set_log_level(Level::TRACE);
}
