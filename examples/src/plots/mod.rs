use tracing_appender::*;
use tracing_error::*;
use tracing_forest::*;
use tracing_subscriber::*;
use tracing_tracy::*;

use ::tracing_forest::util::LevelFilter;
use ::tracing_subscriber::{fmt::Layer, registry, Registry};
use rand::Rng;
use std::thread::sleep;
use std::time::Duration;
use tracing::instrument;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{filter, fmt, prelude::*, reload};
use tracy_client::{Client, PlotConfiguration, PlotFormat, PlotLineStyle, PlotName};

// Plots you know statically can be defined freely like so
const PLOT_PLAYER_COUNT: PlotName = tracy_client::plot_name!("Player Count");
const PLOT_DISK_SPACE: PlotName = tracy_client::plot_name!("Disk Space");

#[tracing::instrument]
pub fn main() {
    let layer = Layer::new().pretty().with_ansi(true);
    let filtered_layer = fmt::Layer::pretty(layer).with_filter(LevelFilter::ERROR);
    let (filtered_layer, reload_handle) = reload::Layer::new(filtered_layer);
    let tracy_layer = tracing_tracy::DefaultConfig::default();
    let tracy_layer = tracing_tracy::TracyLayer::new(tracy_layer);
    registry().with(filtered_layer).with(tracy_layer).init();

    let client = Client::start();
    let mut rng = rand::thread_rng();

    // Anything at runtime needs to be created via PlotName
    let bandwidth = PlotName::new_leak("Bandwidth".to_string());

    // You can configure how plots display, this only needs to be done once
    client.plot_config(
        PLOT_DISK_SPACE,
        PlotConfiguration::default()
            .format(PlotFormat::Memory)
            .fill(false),
    );
    client.plot_config(
        bandwidth,
        PlotConfiguration::default()
            .format(PlotFormat::Percentage)
            .color(Some(0xFF0000))
            .line_style(PlotLineStyle::Stepped),
    );

    for _ in 0..50 {
        // You don't need to constantly send a value!
        if rng.gen_bool(0.75) {
            client.plot(PLOT_PLAYER_COUNT, rng.gen_range(0..10) as f64);
        }

        client.plot(PLOT_DISK_SPACE, rng.gen_range(0..1000000) as f64);
        client.plot(bandwidth, rng.gen_range(0..100) as f64);

        sleep(Duration::from_millis(20));
    }
}
