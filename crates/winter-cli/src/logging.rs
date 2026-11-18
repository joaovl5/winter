pub(crate) fn setup_logging() {
  use tracing_subscriber::fmt::time::LocalTime;
  use tracing_subscriber::{EnvFilter, fmt};

  let time_format =
    time::macros::format_description!("[hour]:[minute]:[second]");
  fmt()
    .with_timer(LocalTime::new(time_format))
    .with_env_filter(EnvFilter::from_default_env())
    .compact()
    .init();
}
