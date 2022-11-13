mod drivers;

use std::future::ready;
use std::sync::Arc;
use std::time::Duration;
use tracing::{Level, info, debug, Instrument, instrument, trace_span};
use streamduck_core::devices::drivers::DriverManager;
use streamduck_core::events::{Event, EventDispatcher, EventInstance};
use serde::{Serialize, Deserialize};
use tokio::spawn;
use tokio::time::{Instant, sleep};
use streamduck_core::events::listeners::{EventListener, GeneralListener, ListensFor, SpecificListener};
use streamduck_core::events::util::cast_event;
use streamduck_core::{init_managers, type_of};
use streamduck_core::devices::buttons::{ButtonEvent, ButtonPosition};
use streamduck_core::devices::SharedDevice;
use streamduck_core::image_lib::{DynamicImage, open};
use crate::drivers::load_drivers;

/// the entry point for the streamdeck application
#[tokio::main]
async fn main() {
    // TODO: change filter level depending on flag
    tracing_subscriber::fmt()
        .compact()
        .with_target(true)
        .with_max_level(Level::TRACE)
        .init();

    info!("Starting...");

    let bundle = init_managers()
        .await
        .expect("Failed to initialize managers");

    load_drivers(&bundle).await;

    let device_metadata = bundle.driver_manager().list_devices().await
        .into_iter()
        // .find(|m| m.identifier.contains("AL10J2C00059"))
        .nth(0)
        .expect("Device not found");

    debug!("Device metadata: {:#?}", device_metadata);

    let device = bundle.driver_manager()
        .connect_device(&device_metadata.driver_name, &device_metadata.identifier).await
        .expect("Failed to connect");

    let img = open("technician.jpg").unwrap();
    device.add_image(1, img).await.unwrap();

    let dispatcher = EventDispatcher::new();

    let _listener = dispatcher.add_listener(LightUpListener {
        device: device.clone()
    }).await;

    loop {
        device.poll(&dispatcher)
            .await
            .unwrap();
        sleep(Duration::from_micros(25)).await;
    }
}

pub struct LightUpListener {
    device: SharedDevice
}

#[streamduck_core::async_trait]
impl EventListener for LightUpListener {
    fn listens_for(&self) -> ListensFor {
        ListensFor::Specific(type_of!(ButtonEvent))
    }

    async fn invoke(&self, e: &dyn EventInstance) {
        if let Some(event) = cast_event::<ButtonEvent>(e) {
            match event {
                ButtonEvent::ButtonDown(p) => self.device.set_button_image(p, 1).await.unwrap(),
                ButtonEvent::ButtonUp(p) => self.device.clear_button_image(p).await.unwrap(),
            }
        }
    }
}
