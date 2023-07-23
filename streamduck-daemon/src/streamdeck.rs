use std::collections::HashMap;
use std::fs::read;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, OnceLock};
use elgato_streamdeck::{AsyncStreamDeck, new_hidapi, StreamDeckError};
use elgato_streamdeck::asynchronous::{DeviceStateReader, DeviceStateUpdate, list_devices_async, refresh_device_list_async};
use elgato_streamdeck::info::Kind;
use hidapi::{HidApi, HidError};
use rmpv::Value;
use tokio::sync::RwLock;
use tracing::trace;
use streamduck_core::config::SharedConfig;
use streamduck_core::core::Core;
use streamduck_core::data::Options;
use streamduck_core::device::{Device, DeviceError, DeviceIdentifier, DeviceImpl, DeviceMetadata, Driver, DriverConnection, DriverImpl, ImageOps};
use streamduck_core::device::input::{Input, InputLayout, InputType};
use streamduck_core::device::metadata::{PartialIdentifier, PartialMetadata};
use streamduck_core::event::{Event, INPUT_EVENT_NAME, InputEvent, InputPayload};
use streamduck_core::msgpack;
use streamduck_core::plugin::{Plugin, PluginHolder, SharedPlugin};
use streamduck_core::plugin::builder::PluginBuilder;

static INPUT_LAYOUTS: OnceLock<HashMap<Kind, InputLayout>> = OnceLock::new();

const MAPPING: &[(Kind, &str)] = &[
    (Kind::Original, "Stream Deck Original"),
    (Kind::OriginalV2, "Stream Deck Original V2"),
    (Kind::Mini, "Stream Deck Mini"),
    (Kind::Xl, "Stream Deck XL"),
    (Kind::XlV2, "Stream Deck XL V2"),
    (Kind::Mk2, "Stream Deck MK2"),
    (Kind::MiniMk2, "Stream Deck MK2 Mini"),
    (Kind::Pedal, "Stream Deck Pedal"),
    (Kind::Plus, "Stream Deck Plus"),
];

pub async fn make_streamdeck_plugin(config: SharedConfig) -> Result<SharedPlugin, StreamDeckError> {
    let hid = Arc::new(RwLock::new(new_hidapi()?));

    let plugin = PluginBuilder::new("streamdeck-support", config)
        .add_driver("elgato-streamdeck", StreamdeckDriver {
            hidapi: hid.clone()
        }, Default::default())
        .build();

    Ok(plugin)
}

pub struct StreamdeckDriver {
    hidapi: Arc<RwLock<HidApi>>
}

#[async_trait::async_trait]
impl DriverImpl for StreamdeckDriver {
    async fn options_changed(&self, _options: &Options, new_data: Value) {
        todo!()
    }

    async fn tick(&self, _options: &Options, core: Arc<Core>) {
        // Nothing to do here yet
    }

    async fn list_devices(&self, _options: &Options) -> Vec<PartialIdentifier> {
        let mut hid = self.hidapi.write().await;

        refresh_device_list_async(hid.deref_mut()).ok();

        list_devices_async(hid.deref()).into_iter()
            .map(|(kind, serial)| PartialIdentifier {
                identifier: serial,
                description: kind_to_description(&kind).to_string(),
            })
            .collect()
    }

    async fn describe_device(&self, _options: &Options, device: &DeviceIdentifier) -> PartialMetadata {
        let kind = description_to_kind(&device.description);

        PartialMetadata {
            identifier: device.clone().downgrade(),
            layout: INPUT_LAYOUTS.get_or_init(input_layouts).get(kind).unwrap().clone(),
        }
    }

    async fn default_device_data(&self, _options: &Options, _device: &DeviceIdentifier) -> Value {
        Value::Nil
    }

    async fn connect_device(&self, options: &Options, device: &DeviceIdentifier) -> Result<DriverConnection, DeviceError> {
        let kind = description_to_kind(&device.description);

        let streamdeck = AsyncStreamDeck::connect(
            self.hidapi.read().await.deref(),
            *kind,
            &device.identifier
        ).map_err(map_error)?;
        let reader = streamdeck.get_reader();

        let device = StreamdeckDevice {
            device: streamdeck,
            reader
        };

        Ok(DriverConnection {
            ui: vec![],
            implement: Arc::new(device),
        })
    }
}

pub struct StreamdeckDevice {
    device: AsyncStreamDeck,
    reader: Arc<DeviceStateReader>
}

#[async_trait::async_trait]
impl DeviceImpl for StreamdeckDevice {
    async fn options_changed(&self, options: &Options, new_data: Value) {
        todo!()
    }

    async fn poll_device(&self, options: &Options, core: Arc<Core>) -> Result<(), DeviceError> {
        let config = core.get_config().load();
        let events = self.reader.read(config.tick_rate).await.map_err(map_error)?;

        for event in events {
            for payload in map_update(event) {
                core.dispatcher.send(Event {
                    name: INPUT_EVENT_NAME.to_string(),
                    payload: Value::from(&payload),
                }).await;
            }
        }

        Ok(())
    }

    async fn disconnect(&self) {
        // Nothing to do here
    }
}

#[async_trait::async_trait]
impl ImageOps for StreamdeckDevice {
    async fn hash_exists(&self, options: &Options, key: u128) -> Result<bool, DeviceError> {
        todo!()
    }

    async fn set_image(&self, options: &Options, input: u16, key: u128) -> Result<bool, DeviceError> {
        todo!()
    }

    async fn clear_image(&self, options: &Options, input: u16) -> Result<(), DeviceError> {
        todo!()
    }

    async fn upload_image(&self, options: &Options, key: u128, image: image::DynamicImage) -> Result<(), DeviceError> {
        todo!()
    }
}

fn map_error(err: StreamDeckError) -> DeviceError {
    if let StreamDeckError::HidError(hiderror) = &err {
        if let HidError::HidApiError { message } = &hiderror {
            if message.contains("device disconnected") {
                return DeviceError::LostConnection;
            }
        }
    }

    DeviceError::DeviceError(Box::new(err))
}

fn map_update(update: DeviceStateUpdate) -> Vec<InputPayload> {
    match update {
        DeviceStateUpdate::ButtonDown(input) => {
            vec![InputPayload {
                input: input as u16,
                event: InputEvent::ButtonPressed,
            }]
        }

        DeviceStateUpdate::ButtonUp(input) => {
            vec![InputPayload {
                input: input as u16,
                event: InputEvent::ButtonReleased,
            }]
        }

        DeviceStateUpdate::EncoderDown(input) => {
            vec![InputPayload {
                input: input as u16 + 9,
                event: InputEvent::ButtonPressed,
            }]
        }

        DeviceStateUpdate::EncoderUp(input) => {
            vec![InputPayload {
                input: input as u16 + 9,
                event: InputEvent::ButtonReleased,
            }]
        }

        DeviceStateUpdate::EncoderTwist(input, amount) => {
            vec![InputPayload {
                input: input as u16 + 9,
                event: InputEvent::EndlessKnob(amount as i16)
            }]
        }

        DeviceStateUpdate::TouchScreenPress(x, y) => {
            vec![InputPayload {
                input: 8,
                event: InputEvent::XYPanelPress {
                    position: (x as u32, y as u32),
                }
            }, InputPayload {
                input: 8,
                event: InputEvent::XYPanelRelease {
                    position: (x as u32, y as u32),
                    time_held: 0.2,
                }
            }]
        }

        DeviceStateUpdate::TouchScreenLongPress(x, y) => {
            vec![InputPayload {
                input: 8,
                event: InputEvent::XYPanelPress {
                    position: (x as u32, y as u32),
                }
            }, InputPayload {
                input: 8,
                event: InputEvent::XYPanelRelease {
                    position: (x as u32, y as u32),
                    time_held: 1.1,
                }
            }]
        }

        DeviceStateUpdate::TouchScreenSwipe(s, e) => {
            vec![InputPayload {
                input: 8,
                event: InputEvent::XYPanelSwipe {
                    start: (s.0 as u32, s.1 as u32),
                    end: (e.0 as u32, e.1 as u32),
                },
            }]
        }
    }
}

fn kind_to_description(kind: &Kind) -> &str {
    for (k, s) in MAPPING {
        if k == kind {
            return s;
        }
    }

    "Unknown" // Usually doesn't happen
}

fn description_to_kind(desc: &str) -> &Kind {
    for (k, s) in MAPPING {
        if *s == desc{
            return k;
        }
    }

    panic!("how could this description end up here? '{}'", desc) // Usually doesn't happen
}

fn input_layouts() -> HashMap<Kind, InputLayout> {
    HashMap::from([
        buttons_only(Kind::Original),
        buttons_only(Kind::OriginalV2),
        buttons_only(Kind::Mk2),
        buttons_only(Kind::Mini),
        buttons_only(Kind::MiniMk2),
        buttons_only(Kind::Xl),
        buttons_only(Kind::XlV2),
        pedal(),
        plus()
    ])
}

fn buttons_only(kind: Kind) -> (Kind, InputLayout) {
    let resolution = kind.key_image_format().size;
    let mut vec = Vec::new();

    for y in 0..kind.row_count() {
        for x in 0..kind.column_count() {
            vec.push(Input {
                x: x as i32,
                y: y as i32,
                w: 1,
                h: 1,
                ty: InputType::Button,
                resolution: Some((resolution.0 as u32, resolution.1 as u32)),
                trigger_presets: vec![],
            })
        }
    }

    (kind, vec)
}

fn pedal() -> (Kind, InputLayout) {
    let kind = Kind::Pedal;
    let mut vec = Vec::new();

    for i in 0..kind.column_count() {
        vec.push(Input {
            x: i as i32,
            y: 0,
            w: 1,
            h: 1,
            ty: InputType::Button,
            resolution: None,
            trigger_presets: vec![],
        })
    }

    (kind, vec)
}

fn plus() -> (Kind, InputLayout) {
    let kind = Kind::Plus;

    let (_, mut vec) = buttons_only(kind);

    // LCD screen
    let lcd_resolution = kind.lcd_strip_size().unwrap();

    vec.push(Input {
        x: 0,
        y: 3,
        w: 4,
        h: 1,
        ty: InputType::XYPanel,
        resolution: Some((lcd_resolution.0 as u32, lcd_resolution.1 as u32)),
        trigger_presets: vec![],
    });

    // Encoders
    for i in 0..kind.encoder_count() {
        vec.push(Input {
            x: i as i32,
            y: 4,
            w: 1,
            h: 1,
            ty: InputType::EndlessKnob,
            resolution: None,
            trigger_presets: vec![],
        })
    }

    (kind, vec)
}