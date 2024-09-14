use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub(crate) fn trace_init() {
	tracing_subscriber::registry().with(fmt::layer()).init();
}