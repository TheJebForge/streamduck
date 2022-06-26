//! Requests related to buttons
use serde::{Deserialize, Serialize};
use streamduck_core::core::button::Button;
use streamduck_core::core::CoreHandle;
use streamduck_core::modules::components::UIPathValue;
use streamduck_core::socket::{check_packet_for_data, parse_packet_to_data, send_packet, SocketData, SocketHandle, SocketPacket};
use streamduck_core::util::{button_to_raw, make_button_unique};
use crate::daemon_data::{DaemonListener, DaemonRequest};
use std::ops::Deref;
use streamduck_core::async_trait;

/// Request for getting a button from current screen on a device
#[derive(Serialize, Deserialize)]
pub struct GetButton {
    pub serial_number: String,
    pub key: u8
}

/// Response of [GetButton] request
#[derive(Serialize, Deserialize)]
pub enum GetButtonResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if there's no button there
    NoButton,

    /// Sent if successfully got the button
    Button(Button)
}

impl SocketData for GetButton {
    const NAME: &'static str = "get_button";
}

impl SocketData for GetButtonResult {
    const NAME: &'static str = "get_button";
}

#[async_trait]
impl DaemonRequest for GetButton {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<GetButton>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number).await {
                let wrapped_core = CoreHandle::wrap(device.core);

                if let Some(button) = wrapped_core.get_button(request.key).await {
                    send_packet(handle, packet, &GetButtonResult::Button(button_to_raw(&button).await)).await.ok();
                } else {
                    send_packet(handle, packet, &GetButtonResult::NoButton).await.ok();
                }
            } else {
                send_packet(handle, packet, &GetButtonResult::DeviceNotFound).await.ok();
            }
        }
    }
}

/// Request for setting a button on current screen on a device
#[derive(Serialize, Deserialize)]
pub struct SetButton {
    pub serial_number: String,
    pub key: u8,
    pub button: Button
}

/// Response of [SetButton] request
#[derive(Serialize, Deserialize)]
pub enum SetButtonResult {
    /// Sent if there's no screen to set to
    NoScreen,

    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if successfully set the button
    Set
}

impl SocketData for SetButton {
    const NAME: &'static str = "set_button";
}

impl SocketData for SetButtonResult {
    const NAME: &'static str = "set_button";
}

#[async_trait]
impl DaemonRequest for SetButton {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<SetButton>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number).await {
                let wrapped_core = CoreHandle::wrap(device.core);

                if wrapped_core.set_button(request.key, make_button_unique(request.button)).await {
                    send_packet(handle, packet, &SetButtonResult::Set).await.ok();
                } else {
                    send_packet(handle, packet, &SetButtonResult::NoScreen).await.ok();
                }
            } else {
                send_packet(handle, packet, &SetButtonResult::DeviceNotFound).await.ok();
            }
        }
    }
}

/// Request for clearing a button on current screen on a device
#[derive(Serialize, Deserialize)]
pub struct ClearButton {
    pub serial_number: String,
    pub key: u8,
}

/// Response of [ClearButton] request
#[derive(Serialize, Deserialize)]
pub enum ClearButtonResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if there's no screen, or there's no button to clear
    FailedToClear,

    /// Sent if successfully set the button
    Cleared
}

impl SocketData for ClearButton {
    const NAME: &'static str = "clear_button";
}

impl SocketData for ClearButtonResult {
    const NAME: &'static str = "clear_button";
}

#[async_trait]
impl DaemonRequest for ClearButton {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<ClearButton>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number).await {
                let wrapped_core = CoreHandle::wrap(device.core);

                if wrapped_core.clear_button(request.key).await {
                    send_packet(handle, packet, &ClearButtonResult::Cleared).await.ok();
                } else {
                    send_packet(handle, packet, &ClearButtonResult::FailedToClear).await.ok();
                }
            } else {
                send_packet(handle, packet, &ClearButtonResult::DeviceNotFound).await.ok();
            }
        }
    }
}

/// Request for adding a new empty button
#[derive(Serialize, Deserialize)]
pub struct NewButton {
    pub serial_number: String,
    pub key: u8,
}

/// Response of [NewButton] request
#[derive(Serialize, Deserialize)]
pub enum NewButtonResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if button failed to be created on specified spot
    FailedToCreate,

    /// Sent if button was successfully created
    Created,
}

impl SocketData for NewButton {
    const NAME: &'static str = "new_button";
}

impl SocketData for NewButtonResult {
    const NAME: &'static str = "new_button";
}

#[async_trait]
impl DaemonRequest for NewButton {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<NewButton>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number).await {
                let wrapped_core = CoreHandle::wrap(device.core);

                if wrapped_core.set_button(request.key, make_button_unique(Button::new())).await {
                    send_packet(handle, packet, &NewButtonResult::Created).await.ok();
                } else {
                    send_packet(handle, packet, &NewButtonResult::FailedToCreate).await.ok();
                }
            } else {
                send_packet(handle, packet, &NewButtonResult::DeviceNotFound).await.ok();
            }
        }
    }
}

/// Request for adding a new button from specified component
#[derive(Serialize, Deserialize)]
pub struct NewButtonFromComponent {
    pub serial_number: String,
    pub key: u8,
    pub component_name: String,
}

/// Response of [NewButtonFromComponent] request
#[derive(Serialize, Deserialize)]
pub enum NewButtonFromComponentResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if component wasn't found
    ComponentNotFound,

    /// Sent if button failed to be created on specified spot
    FailedToCreate,

    /// Sent if button was successfully created
    Created,
}

impl SocketData for NewButtonFromComponent {
    const NAME: &'static str = "new_button_from_component";
}

impl SocketData for NewButtonFromComponentResult {
    const NAME: &'static str = "new_button_from_component";
}

#[async_trait]
impl DaemonRequest for NewButtonFromComponent {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<NewButtonFromComponent>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number).await {
                let wrapped_core = CoreHandle::wrap(device.core);

                let map = listener.module_manager.read_component_map().await;

                if let Some((definition, module)) = map.get(&request.component_name).cloned() {
                    drop(map);

                    let mut button = Button::new();
                    button.insert_component(definition.default_looks).ok();

                    module.add_component(wrapped_core.clone_for(&module), &mut button, &request.component_name).await;

                    if wrapped_core.set_button(request.key, make_button_unique(button)).await {
                        send_packet(handle, packet, &NewButtonFromComponentResult::Created).await.ok();
                    } else {
                        send_packet(handle, packet, &NewButtonFromComponentResult::FailedToCreate).await.ok();
                    }

                    return;
                }

                send_packet(handle, packet, &NewButtonFromComponentResult::ComponentNotFound).await.ok();
            } else {
                send_packet(handle, packet, &NewButtonFromComponentResult::DeviceNotFound).await.ok();
            }
        }
    }
}

// Components
/// Request for adding components onto buttons
#[derive(Serialize, Deserialize)]
pub struct AddComponent {
    pub serial_number: String,
    pub key: u8,
    pub component_name: String,
}

/// Response of [AddComponent] request
#[derive(Serialize, Deserialize)]
pub enum AddComponentResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if failed to add component
    FailedToAdd,

    /// Sent if component was successfully added
    Added,
}

impl SocketData for AddComponent {
    const NAME: &'static str = "add_component";
}

impl SocketData for AddComponentResult {
    const NAME: &'static str = "add_component";
}

#[async_trait]
impl DaemonRequest for AddComponent {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<AddComponent>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number).await {
                let wrapped_core = CoreHandle::wrap(device.core);

                if wrapped_core.add_component(request.key, &request.component_name).await {
                    send_packet(handle, packet, &AddComponentResult::Added).await.ok();
                } else {
                    send_packet(handle, packet, &AddComponentResult::FailedToAdd).await.ok();
                }
            } else {
                send_packet(handle, packet, &AddComponentResult::DeviceNotFound).await.ok();
            }
        }
    }
}

/// Request for adding components onto buttons
#[derive(Serialize, Deserialize)]
pub struct GetComponentValues {
    pub serial_number: String,
    pub key: u8,
    pub component_name: String,
}

/// Response of [GetComponentValues] request
#[derive(Serialize, Deserialize)]
pub enum GetComponentValuesResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if failed to get component values
    FailedToGet,

    /// Sent if component values were successfully retrieved
    Values(Vec<UIPathValue>),
}

impl SocketData for GetComponentValues {
    const NAME: &'static str = "get_component_values";
}

impl SocketData for GetComponentValuesResult {
    const NAME: &'static str = "get_component_values";
}

#[async_trait]
impl DaemonRequest for GetComponentValues {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<GetComponentValues>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number).await {
                let wrapped_core = CoreHandle::wrap(device.core);

                let values = wrapped_core.get_component_values_with_paths(request.key, &request.component_name).await;

                if let Some(values) = values {
                    send_packet(handle, packet, &GetComponentValuesResult::Values(values)).await.ok();
                } else {
                    send_packet(handle, packet, &GetComponentValuesResult::FailedToGet).await.ok();
                }
            } else {
                send_packet(handle, packet, &GetComponentValuesResult::DeviceNotFound).await.ok();
            }
        }
    }
}

/// Request for adding element into component value array
#[derive(Serialize, Deserialize)]
pub struct AddComponentValue {
    pub serial_number: String,
    pub key: u8,
    pub component_name: String,
    pub path: String,
}

/// Response of [AddComponentValue] request
#[derive(Serialize, Deserialize)]
pub enum AddComponentValueResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if failed to add component parameter
    FailedToAdd,

    /// Sent if component value was successfully added
    Added,
}

impl SocketData for AddComponentValue {
    const NAME: &'static str = "add_component_value";
}

impl SocketData for AddComponentValueResult {
    const NAME: &'static str = "add_component_value";
}

#[async_trait]
impl DaemonRequest for AddComponentValue {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<AddComponentValue>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number).await {
                let wrapped_core = CoreHandle::wrap(device.core);

                if wrapped_core.add_element_component_value(request.key, &request.component_name, &request.path).await {
                    listener.config.sync_images(&request.serial_number).await;
                    send_packet(handle, packet, &AddComponentValueResult::Added).await.ok();
                } else {
                    send_packet(handle, packet, &AddComponentValueResult::FailedToAdd).await.ok();
                }
            } else {
                send_packet(handle, packet, &AddComponentValueResult::DeviceNotFound).await.ok();
            }
        }
    }
}

/// Request for removing element from component value array
#[derive(Serialize, Deserialize)]
pub struct RemoveComponentValue {
    pub serial_number: String,
    pub key: u8,
    pub component_name: String,
    pub path: String,
    pub index: usize,
}

/// Response of [RemoveComponentValue] request
#[derive(Serialize, Deserialize)]
pub enum RemoveComponentValueResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if failed to remove component parameter
    FailedToRemove,

    /// Sent if component value was successfully removed
    Removed,
}

impl SocketData for RemoveComponentValue {
    const NAME: &'static str = "remove_component_value";
}

impl SocketData for RemoveComponentValueResult {
    const NAME: &'static str = "remove_component_value";
}

#[async_trait]
impl DaemonRequest for RemoveComponentValue {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<RemoveComponentValue>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number).await {
                let wrapped_core = CoreHandle::wrap(device.core);

                if wrapped_core.remove_element_component_value(request.key, &request.component_name, &request.path, request.index).await {
                    listener.config.sync_images(&request.serial_number).await;
                    send_packet(handle, packet, &RemoveComponentValueResult::Removed).await.ok();
                } else {
                    send_packet(handle, packet, &RemoveComponentValueResult::FailedToRemove).await.ok();
                }
            } else {
                send_packet(handle, packet, &RemoveComponentValueResult::DeviceNotFound).await.ok();
            }
        }
    }
}

/// Request for setting component value
#[derive(Serialize, Deserialize)]
pub struct SetComponentValue {
    pub serial_number: String,
    pub key: u8,
    pub component_name: String,
    pub value: UIPathValue,
}

/// Response of [SetComponentValue] request
#[derive(Serialize, Deserialize)]
pub enum SetComponentValueResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if failed to set component parameter
    FailedToSet,

    /// Sent if component value was successfully set
    Set,
}

impl SocketData for SetComponentValue {
    const NAME: &'static str = "set_component_value";
}

impl SocketData for SetComponentValueResult {
    const NAME: &'static str = "set_component_value";
}

#[async_trait]
impl DaemonRequest for SetComponentValue {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<SetComponentValue>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number).await {
                let wrapped_core = CoreHandle::wrap(device.core);

                if wrapped_core.set_component_value_by_path(request.key, &request.component_name, request.value).await {
                    listener.config.sync_images(&request.serial_number).await;
                    send_packet(handle, packet, &SetComponentValueResult::Set).await.ok();
                } else {
                    send_packet(handle, packet, &SetComponentValueResult::FailedToSet).await.ok();
                }
            } else {
                send_packet(handle, packet, &SetComponentValueResult::DeviceNotFound).await.ok();
            }
        }
    }
}

/// Request for adding components onto buttons
#[derive(Serialize, Deserialize)]
pub struct RemoveComponent {
    pub serial_number: String,
    pub key: u8,
    pub component_name: String,
}

/// Response of [RemoveComponent] request
#[derive(Serialize, Deserialize)]
pub enum RemoveComponentResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if failed to remove component
    FailedToRemove,

    /// Sent if component value was successfully set
    Removed,
}

impl SocketData for RemoveComponent {
    const NAME: &'static str = "remove_component";
}

impl SocketData for RemoveComponentResult {
    const NAME: &'static str = "remove_component";
}

#[async_trait]
impl DaemonRequest for RemoveComponent {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<RemoveComponent>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number).await {
                let wrapped_core = CoreHandle::wrap(device.core);

                if wrapped_core.remove_component(request.key, &request.component_name).await {
                    send_packet(handle, packet, &RemoveComponentResult::Removed).await.ok();
                } else {
                    send_packet(handle, packet, &RemoveComponentResult::FailedToRemove).await.ok();
                }
            } else {
                send_packet(handle, packet, &RemoveComponentResult::DeviceNotFound).await.ok();
            }
        }
    }
}

/// Request for checking clipboard status
#[derive(Serialize, Deserialize)]
pub enum ClipboardStatusResult {
    /// Sent if clipboard is empty
    Empty,

    /// Sent if clipboard has anything
    Full,
}

impl SocketData for ClipboardStatusResult {
    const NAME: &'static str = "clipboard_status";
}

#[async_trait]
impl DaemonRequest for ClipboardStatusResult {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if check_packet_for_data::<ClipboardStatusResult>(packet) {
            let lock = listener.clipboard.lock().await;

            send_packet(handle, packet, &if lock.is_some() { ClipboardStatusResult::Full } else { ClipboardStatusResult::Empty }).await.ok();
        }
    }
}


/// Request to copy a button
#[derive(Serialize, Deserialize)]
pub struct CopyButton {
    pub serial_number: String,
    pub key: u8,
}

/// Response of [CopyButton] request
#[derive(Serialize, Deserialize)]
pub enum CopyButtonResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if there's no button to copy
    NoButton,

    /// Sent if successfully copied a button
    Copied
}

impl SocketData for CopyButton {
    const NAME: &'static str = "copy_button";
}

impl SocketData for CopyButtonResult {
    const NAME: &'static str = "copy_button";
}

#[async_trait]
impl DaemonRequest for CopyButton {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<CopyButton>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number).await {
                let wrapped_core = CoreHandle::wrap(device.core);

                if let Some(button) = wrapped_core.get_button(request.key).await {
                    let mut lock = listener.clipboard.lock().await;
                    *lock = Some(button.read().await.deref().clone());
                    send_packet(handle, packet, &CopyButtonResult::Copied).await.ok();
                } else {
                    send_packet(handle, packet, &CopyButtonResult::NoButton).await.ok();
                }
            } else {
                send_packet(handle, packet, &CopyButtonResult::DeviceNotFound).await.ok();
            }
        }
    }
}

/// Request for pasting button
#[derive(Serialize, Deserialize)]
pub struct PasteButton {
    pub serial_number: String,
    pub key: u8,
}

/// Response of [PasteButton] request
#[derive(Serialize, Deserialize)]
pub enum PasteButtonResult {
    /// Sent if device wasn't found
    DeviceNotFound,

    /// Sent if failed to paste
    FailedToPaste,

    /// Sent if successfully pasted button
    Pasted
}

impl SocketData for PasteButton {
    const NAME: &'static str = "paste_button";
}

impl SocketData for PasteButtonResult {
    const NAME: &'static str = "paste_button";
}

#[async_trait]
impl DaemonRequest for PasteButton {
    async fn process(listener: &DaemonListener, handle: SocketHandle<'_>, packet: &SocketPacket) {
        if let Ok(request) = parse_packet_to_data::<PasteButton>(packet) {
            if let Some(device) = listener.core_manager.get_device(&request.serial_number).await {
                let wrapped_core = CoreHandle::wrap(device.core);

                let clipboard = listener.clipboard.lock().await;

                if clipboard.is_some() {
                    if wrapped_core.paste_button(request.key, clipboard.as_ref().unwrap()).await {
                        send_packet(handle, packet, &PasteButtonResult::Pasted).await.ok();
                        return;
                    }
                }

                send_packet(handle, packet, &PasteButtonResult::FailedToPaste).await.ok();
            } else {
                send_packet(handle, packet, &PasteButtonResult::DeviceNotFound).await.ok();
            }
        }
    }
}