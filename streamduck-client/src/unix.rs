use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;
use std::sync::{Arc, RwLock};
use serde::{Serialize};
use serde::de::DeserializeOwned;
use streamduck_core::core::button::Button;
use streamduck_core::core::RawButtonPanel;
use streamduck_core::modules::components::{ComponentDefinition, UIValue};
use streamduck_core::modules::PluginMetadata;
use streamduck_core::versions::SOCKET_API;
use streamduck_daemon::socket::daemon_data::{AddDevice, AddDeviceResult, Device, ListDevices, GetDevice, GetDeviceResult, RemoveDevice, RemoveDeviceResult, SocketAPIVersion, ReloadDeviceConfigsResult, ReloadDeviceConfigResult, SaveDeviceConfigsResult, SaveDeviceConfigResult, SetBrightnessResult, ReloadDeviceConfig, SaveDeviceConfig, SetBrightness, ListModules, ListComponents, GetButtonResult, SetButtonResult, ClearButtonResult, PushScreenResult, PopScreenResult, ReplaceScreenResult, ResetStackResult, CommitChangesToConfigResult, GetStackResult, GetCurrentScreenResult, GetStack, GetCurrentScreen, GetButton, SetButton, ClearButton, PushScreen, PopScreen, ReplaceScreen, ResetStack, CommitChangesToConfig, DoButtonActionResult, DoButtonAction, ForciblyPopScreenResult, ForciblyPopScreen, AddComponentResult, GetComponentValuesResult, SetComponentValueResult, RemoveComponentResult, AddComponent, GetComponentValues, SetComponentValue, RemoveComponent};
use streamduck_daemon::socket::{parse_packet_to_data, send_no_data_packet_with_requester, send_packet_with_requester, SocketData, SocketPacket};
use crate::{SDClient, SDClientError};
use std::io::Write;

/// Definition of Unix Socket based client
pub struct UnixClient {
    connection: RwLock<BufReader<UnixStream>>
}

#[allow(dead_code)]
impl UnixClient {
    /// Initializes client using unix domain socket
    pub fn new() -> Result<Arc<Box<dyn SDClient>>, std::io::Error> {
        let client: Arc<Box<dyn SDClient>> = Arc::new(Box::new(UnixClient {
            connection: RwLock::new(BufReader::new(UnixStream::connect("/tmp/streamduck.sock")?))
        }));

        let daemon_version = client.version().expect("Failed to retrieve version");

        if daemon_version != SOCKET_API.1 {
            println!("[Warning] Version of client library doesn't match daemon API version. Client: {}, Daemon: {}", SOCKET_API.1, daemon_version);
        }

        Ok(client)
    }

    fn process_request<Req: SocketData + Serialize, Res: SocketData + DeserializeOwned>(&self, request: &Req) -> Result<Res, SDClientError> {
        let mut handle = self.connection.write().unwrap();

        send_packet_with_requester(handle.get_mut(), "", request)?;

        let mut line = String::new();
        handle.read_line(&mut line)?;

        let packet: SocketPacket = serde_json::from_str(&line)?;

        Ok(parse_packet_to_data(&packet)?)
    }

    fn process_request_without_data<Res: SocketData + DeserializeOwned>(&self) -> Result<Res, SDClientError> {
        let mut handle = self.connection.write().unwrap();

        send_no_data_packet_with_requester::<Res>(handle.get_mut(), "")?;

        let mut line = String::new();
        handle.read_line(&mut line)?;

        let packet: SocketPacket = serde_json::from_str(&line)?;

        Ok(parse_packet_to_data(&packet)?)
    }
}

impl SDClient for UnixClient {
    fn version(&self) -> Result<String, SDClientError> {
        let response: SocketAPIVersion = self.process_request_without_data()?;

        Ok(response.version)
    }

    fn device_list(&self) -> Result<Vec<Device>, SDClientError> {
        let response: ListDevices = self.process_request_without_data()?;

        Ok(response.devices)
    }

    fn get_device(&self, serial_number: &str) -> Result<GetDeviceResult, SDClientError> {
        let response: GetDeviceResult = self.process_request(&GetDevice {
            serial_number: serial_number.to_string()
        })?;

        Ok(response)
    }

    fn add_device(&self, serial_number: &str) -> Result<AddDeviceResult, SDClientError> {
        let response: AddDeviceResult = self.process_request(&AddDevice {
            serial_number: serial_number.to_string()
        })?;

        Ok(response)
    }

    fn remove_device(&self, serial_number: &str) -> Result<RemoveDeviceResult, SDClientError> {
        let response: RemoveDeviceResult = self.process_request(&RemoveDevice {
            serial_number: serial_number.to_string()
        })?;

        Ok(response)
    }

    fn reload_device_configs(&self) -> Result<ReloadDeviceConfigsResult, SDClientError> {
        let response: ReloadDeviceConfigsResult = self.process_request_without_data()?;

        Ok(response)
    }

    fn reload_device_config(&self, serial_number: &str) -> Result<ReloadDeviceConfigResult, SDClientError> {
        let response: ReloadDeviceConfigResult = self.process_request(&ReloadDeviceConfig {
            serial_number: serial_number.to_string()
        })?;

        Ok(response)
    }

    fn save_device_configs(&self) -> Result<SaveDeviceConfigsResult, SDClientError> {
        let response: SaveDeviceConfigsResult = self.process_request_without_data()?;

        Ok(response)
    }

    fn save_device_config(&self, serial_number: &str) -> Result<SaveDeviceConfigResult, SDClientError> {
        let response: SaveDeviceConfigResult = self.process_request(&SaveDeviceConfig {
            serial_number: serial_number.to_string()
        })?;

        Ok(response)
    }

    fn set_brightness(&self, serial_number: &str, brightness: u8) -> Result<SetBrightnessResult, SDClientError> {
        let response: SetBrightnessResult = self.process_request(&SetBrightness {
            serial_number: serial_number.to_string(),
            brightness
        })?;

        Ok(response)
    }

    fn list_modules(&self) -> Result<Vec<PluginMetadata>, SDClientError> {
        let response: ListModules = self.process_request_without_data()?;

        Ok(response.modules)
    }

    fn list_components(&self) -> Result<HashMap<String, HashMap<String, ComponentDefinition>>, SDClientError> {
        let response: ListComponents = self.process_request_without_data()?;

        Ok(response.components)
    }

    fn get_stack(&self, serial_number: &str) -> Result<GetStackResult, SDClientError> {
        let response: GetStackResult = self.process_request(&GetStack {
            serial_number: serial_number.to_string()
        })?;

        Ok(response)
    }

    fn get_current_screen(&self, serial_number: &str) -> Result<GetCurrentScreenResult, SDClientError> {
        let response: GetCurrentScreenResult = self.process_request(&GetCurrentScreen {
            serial_number: serial_number.to_string()
        })?;

        Ok(response)
    }

    fn get_button(&self, serial_number: &str, key: u8) -> Result<GetButtonResult, SDClientError> {
        let response: GetButtonResult = self.process_request(&GetButton {
            serial_number: serial_number.to_string(),
            key
        })?;

        Ok(response)
    }

    fn set_button(&self, serial_number: &str, key: u8, button: Button) -> Result<SetButtonResult, SDClientError> {
        let response: SetButtonResult = self.process_request(&SetButton {
            serial_number: serial_number.to_string(),
            key,
            button
        })?;

        Ok(response)
    }

    fn clear_button(&self, serial_number: &str, key: u8) -> Result<ClearButtonResult, SDClientError> {
        let response: ClearButtonResult = self.process_request(&ClearButton {
            serial_number: serial_number.to_string(),
            key
        })?;

        Ok(response)
    }

    fn add_component(&self, serial_number: &str, key: u8, component_name: &str) -> Result<AddComponentResult, SDClientError> {
        let response: AddComponentResult = self.process_request(&AddComponent {
            serial_number: serial_number.to_string(),
            key,
            component_name: component_name.to_string()
        })?;

        Ok(response)
    }

    fn get_component_values(&self, serial_number: &str, key: u8, component_name: &str) -> Result<GetComponentValuesResult, SDClientError> {
        let response: GetComponentValuesResult = self.process_request(&GetComponentValues {
            serial_number: serial_number.to_string(),
            key,
            component_name: component_name.to_string()
        })?;

        Ok(response)
    }

    fn set_component_values(&self, serial_number: &str, key: u8, component_name: &str, value: UIValue) -> Result<SetComponentValueResult, SDClientError> {
        let response: SetComponentValueResult = self.process_request(&SetComponentValue {
            serial_number: serial_number.to_string(),
            key,
            component_name: component_name.to_string(),
            value
        })?;

        Ok(response)
    }

    fn remove_component(&self, serial_number: &str, key: u8, component_name: &str) -> Result<RemoveComponentResult, SDClientError> {
        let response: RemoveComponentResult = self.process_request(&RemoveComponent {
            serial_number: serial_number.to_string(),
            key,
            component_name: component_name.to_string()
        })?;

        Ok(response)
    }

    fn push_screen(&self, serial_number: &str, screen: RawButtonPanel) -> Result<PushScreenResult, SDClientError> {
        let response: PushScreenResult = self.process_request(&PushScreen {
            serial_number: serial_number.to_string(),
            screen
        })?;

        Ok(response)
    }

    fn pop_screen(&self, serial_number: &str) -> Result<PopScreenResult, SDClientError> {
        let response: PopScreenResult = self.process_request(&PopScreen {
            serial_number: serial_number.to_string()
        })?;

        Ok(response)
    }

    fn forcibly_pop_screen(&self, serial_number: &str) -> Result<ForciblyPopScreenResult, SDClientError> {
        let response: ForciblyPopScreenResult = self.process_request(&ForciblyPopScreen {
            serial_number: serial_number.to_string()
        })?;

        Ok(response)
    }

    fn replace_screen(&self, serial_number: &str, screen: RawButtonPanel) -> Result<ReplaceScreenResult, SDClientError> {
        let response: ReplaceScreenResult = self.process_request(&ReplaceScreen {
            serial_number: serial_number.to_string(),
            screen
        })?;

        Ok(response)
    }

    fn reset_stack(&self, serial_number: &str, screen: RawButtonPanel) -> Result<ResetStackResult, SDClientError> {
        let response: ResetStackResult = self.process_request(&ResetStack {
            serial_number: serial_number.to_string(),
            screen
        })?;

        Ok(response)
    }

    fn commit_changes(&self, serial_number: &str) -> Result<CommitChangesToConfigResult, SDClientError> {
        let response: CommitChangesToConfigResult = self.process_request(&CommitChangesToConfig {
            serial_number: serial_number.to_string()
        })?;

        Ok(response)
    }

    fn do_button_action(&self, serial_number: &str, key: u8) -> Result<DoButtonActionResult, SDClientError> {
        let response: DoButtonActionResult = self.process_request(&DoButtonAction {
            serial_number: serial_number.to_string(),
            key
        })?;

        Ok(response)
    }

    fn send_packet(&self, packet: SocketPacket) -> Result<SocketPacket, SDClientError> {
        let mut handle = self.connection.write().unwrap();
        writeln!(handle.get_mut(), "{}", serde_json::to_string(&packet)?)?;

        let mut line = String::new();
        handle.read_line(&mut line)?;

        Ok(serde_json::from_str(&line)?)
    }

    fn send_packet_without_response(&self, packet: SocketPacket) -> Result<(), SDClientError> {
        let mut handle = self.connection.write().unwrap();
        writeln!(handle.get_mut(), "{}", serde_json::to_string(&packet)?)?;
        Ok(())
    }
}