/// Definitions of button structs
pub mod button;

/// Methods for interacting with the core
pub mod methods;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::mpsc::{channel, Receiver};
use streamdeck::StreamDeck;
use crate::core::button::Button;
use crate::core::methods::{button_down, button_up, CoreHandle};
use crate::modules::ModuleManager;
use crate::threads::rendering::{RendererHandle, spawn_rendering_thread};
use crate::threads::streamdeck::{spawn_streamdeck_thread, StreamDeckCommand, StreamDeckHandle};

/// Reference counted RwLock of a button, prevents data duplication and lets you edit buttons if they're in many stacks at once
pub type UniqueButton = Arc<RwLock<Button>>;

/// Hashmap of UniqueButtons
pub type ButtonPanel = HashMap<u8, UniqueButton>;

/// Hashmap of raw Buttons
pub type RawButtonPanel = HashMap<u8, Button>;

/// Core struct that contains all relevant information about streamdeck and manages the streamdeck
#[allow(dead_code)]
pub struct SDCore {
    /// Module manager
    pub module_manager: Arc<ModuleManager>,

    /// Current panel stack
    pub current_stack: Mutex<Vec<ButtonPanel>>,

    /// Image size supported by streamdeck
    pub image_size: (usize, usize),

    /// Key count of the streamdeck device
    pub key_count: u8,

    /// Pool rate of how often should the core read events from the device
    pub pool_rate: u32,

    /// Decides if core is dead
    pub should_close: RwLock<bool>,

    handles: Mutex<Option<ThreadHandles>>
}

impl SDCore {
    /// Creates an instance of core that is already dead
    pub fn blank(module_manager: Arc<ModuleManager>) -> Arc<SDCore> {
        Arc::new(SDCore {
            module_manager,
            current_stack: Mutex::new(vec![]),
            handles: Mutex::new(None),
            image_size: (0, 0),
            key_count: 0,
            pool_rate: 0,
            should_close: RwLock::new(true)
        })
    }

    /// Creates an instance of the core over existing streamdeck connection
    pub fn new(module_manager: Arc<ModuleManager>, connection: StreamDeck, pool_rate: u32) -> (Arc<SDCore>, KeyHandler) {
        let (key_tx, key_rx) = channel();

        let core = Arc::new(SDCore {
            module_manager,
            current_stack: Mutex::new(vec![]),
            handles: Mutex::new(None),
            image_size: connection.image_size(),
            key_count: connection.kind().keys(),
            pool_rate,
            should_close: RwLock::new(false)
        });

        let streamdeck = spawn_streamdeck_thread(core.clone(), connection, key_tx);
        let renderer = spawn_rendering_thread(core.clone());

        if let Ok(mut handles) = core.handles.lock() {
            *handles = Some(
                ThreadHandles {
                    streamdeck,
                    renderer
                }
            )
        }

        (core.clone(), KeyHandler {
            core: CoreHandle::wrap(core.clone()),
            receiver: key_rx
        })
    }

    /// Tells renderer's thread it's time to redraw
    pub fn mark_for_redraw(&self) {
        let handles = self.handles.lock().unwrap();

        handles.as_ref().unwrap().renderer.redraw();
    }

    /// Sends commands to streamdeck thread
    pub fn send_commands(&self, commands: Vec<StreamDeckCommand>) {
        let handles = self.handles.lock().unwrap();

        handles.as_ref().unwrap().streamdeck.send(commands);
    }

    /// Checks if core is supposed to be closed
    pub fn is_closed(&self) -> bool {
        *self.should_close.read().unwrap()
    }

    /// Kills the core and all the related threads
    pub fn close(&self) {
        let mut lock = self.should_close.write().unwrap();
        *lock = true;
        self.mark_for_redraw();
    }
}

#[derive(Debug)]
struct ThreadHandles {
    pub streamdeck: StreamDeckHandle,
    pub renderer: RendererHandle
}

/// Routine that acts as a middleman between streamdeck thread and the core, was made as a way to get around Sync restriction
pub struct KeyHandler{
    core: CoreHandle,
    receiver: Receiver<(u8, bool)>
}

impl KeyHandler {
    /// Runs the key handling loop in current thread
    pub fn run_loop(&self) {
        loop {
            if self.core.core().is_closed() {
                break
            }

            if let Ok((key, state)) = self.receiver.recv() {
                if state {
                    button_down(&self.core, key);
                } else {
                    button_up(&self.core, key);
                }
            } else {
                break;
            }
        }
    }
}