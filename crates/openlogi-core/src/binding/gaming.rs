//! `0x8110` button layouts of G-series mice, dumped from real hardware.
//! Slot `n` is spy bit `n` and asset slot `g{n + 1}`.

use super::action::Action;
use super::button::ButtonId;
use crate::config::KeyModifiers;
use crate::device::Capabilities;
use crate::hid::DeviceRoute;

/// One `0x8110` slot of a G-series mouse.
#[derive(Debug, PartialEq, Eq)]
pub struct Slot {
    /// The button on the slot.
    pub button: ButtonId,
    /// Whether the slot sends nothing in host mode unless captured.
    pub host_silent: bool,
    /// The button's stock function on this model, for a button whose stock
    /// function differs per model. `None` takes
    /// [`default_binding`](super::default_binding).
    pub stock: Option<Action>,
}

impl Slot {
    /// A slot that host mode still reports to the OS as a standard HID
    /// button.
    #[must_use]
    pub const fn native(button: ButtonId) -> Self {
        Self {
            button,
            host_silent: false,
            stock: None,
        }
    }

    /// A slot that host mode silences, with the same stock function on every
    /// model.
    #[must_use]
    pub const fn silent(button: ButtonId) -> Self {
        Self {
            button,
            host_silent: true,
            stock: None,
        }
    }

    /// A button of a [`KeyedMouse`]: it types a key chord whose native
    /// effect is `stock`. While its binding is `stock` the chord passes
    /// through untouched, so the default is the mouse's own behaviour.
    #[must_use]
    pub const fn keyed(button: ButtonId, stock: Action) -> Self {
        Self {
            button,
            host_silent: false,
            stock: Some(stock),
        }
    }

    /// A slot that host mode silences, with the stock function `stock` on
    /// this model. OpenLogi performs `stock` while it captures the button.
    #[must_use]
    pub const fn stock(button: ButtonId, stock: Action) -> Self {
        Self {
            button,
            host_silent: true,
            stock: Some(stock),
        }
    }
}

/// One model's button slots.
#[derive(Debug, PartialEq, Eq)]
pub struct GamingLayout {
    /// HID++ model keys sharing this layout.
    pub model_keys: &'static [&'static str],
    /// The slots, in `0x8110` order.
    pub slots: &'static [Slot],
    /// Stock DPI levels, used until the user sets presets.
    pub stock_dpi_presets: &'static [u16],
}

/// G502 LIGHTSPEED (wpid `0x407f`). The extended model id tells the colours
/// apart: `0` is the black unit dumped in #1729, `2` the purple one from
/// #2372, which reports the same wpid, model ids and 11 slots. `1` is the
/// pink one, sold with the purple one. No unit has been dumped. G HUB's
/// `g502_wireless` depot gives all three ids one default configuration of 11
/// inputs, and the same hotspot metadata to `1` and `2` (#2375).
pub const G502_LIGHTSPEED: GamingLayout = GamingLayout {
    model_keys: &["0407f", "1407f", "2407f"],
    slots: &[
        Slot::native(ButtonId::LeftClick),
        Slot::native(ButtonId::RightClick),
        Slot::native(ButtonId::MiddleClick),
        Slot::native(ButtonId::Back),
        Slot::native(ButtonId::Forward),
        Slot::stock(ButtonId::G6, Action::DpiShift),
        Slot::stock(ButtonId::G7, Action::PreviousDpiPreset),
        Slot::stock(ButtonId::G8, Action::NextDpiPreset),
        // G9 cycles the onboard profiles, which has no host equivalent.
        Slot::stock(ButtonId::G9, Action::None),
        Slot::silent(ButtonId::WheelTiltRight),
        Slot::silent(ButtonId::WheelTiltLeft),
    ],
    stock_dpi_presets: &[400, 800, 1600, 3200, 6400],
};

/// G903 LIGHTSPEED (wpid `0x4087`), dumped in #2178. Ambidextrous: the left
/// side pair is Back and Forward, the right side pair is G6 and G7, and the
/// DPI keys are G8 (down, left) and G9 (up, right). Slot 11 never reports.
pub const G903_LIGHTSPEED: GamingLayout = GamingLayout {
    model_keys: &["04087"],
    slots: &[
        Slot::native(ButtonId::LeftClick),
        Slot::native(ButtonId::RightClick),
        Slot::native(ButtonId::MiddleClick),
        Slot::native(ButtonId::Back),
        Slot::native(ButtonId::Forward),
        // The right side pair mirrors the left one.
        Slot::stock(ButtonId::G6, Action::MouseBack),
        Slot::stock(ButtonId::G7, Action::MouseForward),
        Slot::stock(ButtonId::G8, Action::PreviousDpiPreset),
        Slot::stock(ButtonId::G9, Action::NextDpiPreset),
        Slot::silent(ButtonId::WheelTiltLeft),
        Slot::silent(ButtonId::WheelTiltRight),
    ],
    stock_dpi_presets: &[400, 800, 1600, 3200, 6400],
};

/// A gaming mouse whose layout nobody has dumped. Its first five slots are
/// the standard HID buttons, which the OS input hook remaps without host
/// mode. G6 and up differ per model and are not guessed.
///
/// For drawing the Buttons panel only: [`GamingLayout::for_model_key`] never
/// returns it, so the agent captures nothing through it.
pub const STANDARD_BUTTONS: GamingLayout = GamingLayout {
    model_keys: &[],
    slots: &[
        Slot::native(ButtonId::LeftClick),
        Slot::native(ButtonId::RightClick),
        Slot::native(ButtonId::MiddleClick),
        Slot::native(ButtonId::Back),
        Slot::native(ButtonId::Forward),
    ],
    stock_dpi_presets: &[],
};

const LAYOUTS: &[&GamingLayout] = &[&G502_LIGHTSPEED, &G903_LIGHTSPEED];

/// A button of a [`KeyedMouse`] that reaches the host as a key chord rather
/// than as a HID button. `keycode` is the platform-neutral keycode the hook
/// reports (macOS `kVK_*`, which every hook backend translates into).
#[derive(Debug, PartialEq, Eq)]
pub struct KeyButton {
    /// The chord's key.
    pub keycode: u16,
    /// The modifiers the mouse holds with it.
    pub modifiers: KeyModifiers,
    /// The button the chord stands for.
    pub button: ButtonId,
}

/// A non-Logitech mouse, listed by the generic-mouse driver, whose extra
/// buttons type key chords through the mouse's own keyboard collection.
/// The OS hook attributes each chord to its sender, so the same chord typed
/// on a real keyboard is never taken for the button.
#[derive(Debug, PartialEq, Eq)]
pub struct KeyedMouse {
    /// HID vendor id.
    pub vendor_id: u16,
    /// HID product id.
    pub product_id: u16,
    /// The buttons the Buttons panel draws.
    pub layout: GamingLayout,
    /// The chord each key-typing button sends.
    pub key_buttons: &'static [KeyButton],
}

/// macOS virtual keycode of F11 (`kVK_F11`).
const KVK_F11: u16 = 0x67;
/// macOS virtual keycode of Q (`kVK_ANSI_Q`).
const KVK_ANSI_Q: u16 = 0x0c;

/// Baseus BS-007Pro "Editor Pro", over Bluetooth LE. Its side button types
/// F11 (Show Desktop) and the button behind the wheel types Control-Command-Q
/// (Lock Screen); the wheel click is a standard middle button. G6 and G7 are
/// the slots those two key-typing buttons take.
pub const BASEUS_EDITOR_PRO: KeyedMouse = KeyedMouse {
    vendor_id: 0xfff1,
    product_id: 0x0010,
    layout: GamingLayout {
        model_keys: &[],
        slots: &[
            Slot::native(ButtonId::LeftClick),
            Slot::native(ButtonId::RightClick),
            Slot::native(ButtonId::MiddleClick),
            Slot::keyed(ButtonId::G6, Action::ShowDesktop),
            Slot::keyed(ButtonId::G7, Action::LockScreen),
        ],
        stock_dpi_presets: &[],
    },
    key_buttons: &[
        KeyButton {
            keycode: KVK_F11,
            modifiers: KeyModifiers {
                shift: false,
                control: false,
                option: false,
                command: false,
            },
            button: ButtonId::G6,
        },
        KeyButton {
            keycode: KVK_ANSI_Q,
            modifiers: KeyModifiers {
                shift: false,
                control: true,
                option: false,
                command: true,
            },
            button: ButtonId::G7,
        },
    ],
};

const KEYED_MICE: &[&KeyedMouse] = &[&BASEUS_EDITOR_PRO];

impl KeyedMouse {
    /// The keyed mouse behind a generic-mouse route, if it is one.
    #[must_use]
    pub fn for_route(route: &DeviceRoute) -> Option<&'static Self> {
        let (vendor_id, product_id) = route.generic_mouse_ids()?;
        Self::for_ids(vendor_id, product_id)
    }

    /// The keyed mouse with these HID ids, if it is one.
    #[must_use]
    pub fn for_ids(vendor_id: u16, product_id: u16) -> Option<&'static Self> {
        KEYED_MICE
            .iter()
            .copied()
            .find(|mouse| (mouse.vendor_id, mouse.product_id) == (vendor_id, product_id))
    }

    /// The button a key-down of `keycode` with `modifiers` stands for.
    #[must_use]
    pub fn button_for_key(&self, keycode: u16, modifiers: KeyModifiers) -> Option<ButtonId> {
        self.key_buttons
            .iter()
            .find(|key| key.keycode == keycode && key.modifiers == modifiers)
            .map(|key| key.button)
    }
}

/// Whether a device is a G-series mouse, and which buttons it offers. The
/// agent, the tabs and the Buttons panel all ask this one verdict.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GamingButtons {
    /// A model whose `0x8110` layout was dumped: host mode captures every
    /// slot.
    Dumped(&'static GamingLayout),
    /// A device with the gaming tables and no ReprogControls whose layout
    /// nobody has dumped: only [`STANDARD_BUTTONS`], which the OS hook
    /// remaps.
    Standard,
    /// A generic mouse whose extra buttons type key chords; the OS hook
    /// remaps them by sender.
    Keyed(&'static KeyedMouse),
}

impl GamingButtons {
    /// The verdict for `model_key` on `route` with `capabilities`, which is
    /// `None` for a device never probed; a dumped model or a keyed mouse
    /// answers without them. `None` for a device that diverts its controls
    /// through ReprogControls.
    #[must_use]
    pub fn for_device(
        model_key: &str,
        route: Option<&DeviceRoute>,
        capabilities: Option<&Capabilities>,
    ) -> Option<Self> {
        if let Some(mouse) = route.and_then(KeyedMouse::for_route) {
            return Some(Self::Keyed(mouse));
        }
        if let Some(layout) = GamingLayout::for_model_key(model_key) {
            return Some(Self::Dumped(layout));
        }
        capabilities
            .is_some_and(Capabilities::no_button_diversion)
            .then_some(Self::Standard)
    }

    /// The buttons the Buttons panel draws.
    #[must_use]
    pub const fn layout(self) -> &'static GamingLayout {
        match self {
            Self::Dumped(layout) => layout,
            Self::Standard => &STANDARD_BUTTONS,
            Self::Keyed(mouse) => &mouse.layout,
        }
    }
}

impl GamingLayout {
    /// The layout for a HID++ model key.
    #[must_use]
    pub fn for_model_key(model_key: &str) -> Option<&'static Self> {
        LAYOUTS
            .iter()
            .copied()
            .find(|layout| layout.model_keys.contains(&model_key))
    }

    /// The slot of `button`.
    #[must_use]
    pub fn slot_of(&self, button: ButtonId) -> Option<u8> {
        let slot = self.slots.iter().position(|slot| slot.button == button)?;
        u8::try_from(slot).ok()
    }

    /// The button printed `G{number}`.
    #[must_use]
    pub fn button_for_g_number(&self, number: u8) -> Option<ButtonId> {
        let slot = usize::from(number.checked_sub(1)?);
        self.slots.get(slot).map(|slot| slot.button)
    }

    /// The button on each slot, in slot order.
    pub fn buttons(&self) -> impl Iterator<Item = ButtonId> + '_ {
        self.slots.iter().map(|slot| slot.button)
    }

    /// Every slot but the primary clicks.
    pub fn remappable(&self) -> impl Iterator<Item = ButtonId> + '_ {
        self.buttons()
            .filter(|b| !matches!(b, ButtonId::LeftClick | ButtonId::RightClick))
    }

    /// Whether rebinding `button` needs host mode.
    #[must_use]
    pub fn needs_host_mode(&self, button: ButtonId) -> bool {
        self.slot(button).is_some_and(|slot| slot.host_silent)
    }

    /// The stock function of `button` on this model, if it differs per
    /// model. [`default_binding_on`](super::default_binding_on) falls back to
    /// [`default_binding`](super::default_binding) for every other button.
    #[must_use]
    pub fn stock_action(&self, button: ButtonId) -> Option<Action> {
        self.slot(button)?.stock.clone()
    }

    fn slot(&self, button: ButtonId) -> Option<&Slot> {
        self.slots.iter().find(|slot| slot.button == button)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gaming_buttons_name_the_dumped_layout_or_the_standard_buttons() {
        let gaming = Capabilities::from_feature_ids(&[0x2201, 0x8100, 0x8110]);
        let mx = Capabilities::from_feature_ids(&[0x1b04, 0x2201]);
        assert_eq!(
            GamingButtons::for_device("0407f", None, None),
            Some(GamingButtons::Dumped(&G502_LIGHTSPEED)),
            "a dumped model answers before it is probed"
        );
        assert_eq!(
            GamingButtons::for_device("0409f", None, Some(&gaming)),
            Some(GamingButtons::Standard)
        );
        assert_eq!(GamingButtons::Standard.layout(), &STANDARD_BUTTONS);
        assert_eq!(GamingButtons::for_device("0409f", None, None), None);
        assert_eq!(GamingButtons::for_device("2b042", None, Some(&mx)), None);
    }

    #[test]
    fn g502_lightspeed_slots_match_the_hardware_dump() {
        let layout = GamingLayout::for_model_key("0407f").unwrap();
        assert_eq!(layout.slot_of(ButtonId::G9), Some(8));
        assert_eq!(layout.slot_of(ButtonId::WheelTiltLeft), Some(10));
        assert_eq!(layout.button_for_g_number(4), Some(ButtonId::Back));
        assert_eq!(
            layout.button_for_g_number(10),
            Some(ButtonId::WheelTiltRight)
        );
        assert_eq!(layout.button_for_g_number(0), None);
        assert_eq!(layout.button_for_g_number(12), None);
        assert!(!layout.remappable().any(|b| b == ButtonId::LeftClick));
        assert!(layout.needs_host_mode(ButtonId::G7));
        assert!(!layout.needs_host_mode(ButtonId::Back));
    }

    /// #2372: the purple G502 LIGHTSPEED reports extended model id 2, and
    /// its gaming tables made it a five-button `Standard` mouse. #2375: the
    /// pink one has id 1.
    #[test]
    fn every_g502_lightspeed_colour_takes_its_layout() {
        use crate::device::{DeviceModelInfo, DeviceTransports};
        let gaming = Capabilities::from_feature_ids(&[0x2201, 0x8060, 0x8100, 0x8110]);
        for extended_model_id in [0x00, 0x01, 0x02] {
            let model = DeviceModelInfo {
                entity_count: 3,
                serial_number: None,
                unit_id: [0; 4],
                transports: DeviceTransports {
                    usb: true,
                    equad: true,
                    btle: false,
                    bluetooth: false,
                },
                model_ids: [0x407f, 0xc08d, 0x0000],
                extended_model_id,
            };
            assert_eq!(
                GamingButtons::for_device(&model.model_key(), None, Some(&gaming)),
                Some(GamingButtons::Dumped(&G502_LIGHTSPEED)),
                "extended model id {extended_model_id}"
            );
        }
    }

    fn raw_route(vendor_id: u16, product_id: u16, usage_page: u16, usage_id: u16) -> DeviceRoute {
        DeviceRoute::RawHid {
            vendor_id,
            product_id,
            usage_page,
            usage_id,
            identity: "serial:20240500".to_string(),
        }
    }

    #[test]
    fn a_keyed_mouse_route_answers_before_model_key_or_capabilities() {
        let baseus = raw_route(0xfff1, 0x0010, 0x0001, 0x0002);
        assert_eq!(
            GamingButtons::for_device("Baseus Editor Pro", Some(&baseus), None),
            Some(GamingButtons::Keyed(&BASEUS_EDITOR_PRO))
        );
        assert_eq!(
            GamingButtons::Keyed(&BASEUS_EDITOR_PRO).layout(),
            &BASEUS_EDITOR_PRO.layout
        );
        // Another collection of the same mouse is not its mouse route, and an
        // unlisted generic mouse keeps the capability verdict.
        let keyboard = raw_route(0xfff1, 0x0010, 0x0001, 0x0006);
        assert_eq!(GamingButtons::for_device("x", Some(&keyboard), None), None);
        let other = raw_route(0x1234, 0x0010, 0x0001, 0x0002);
        assert_eq!(GamingButtons::for_device("x", Some(&other), None), None);
    }

    #[test]
    fn keyed_mouse_chords_need_their_exact_modifiers() {
        let none = KeyModifiers::default();
        let lock = KeyModifiers {
            control: true,
            command: true,
            ..KeyModifiers::default()
        };
        let baseus = &BASEUS_EDITOR_PRO;
        assert_eq!(baseus.button_for_key(KVK_F11, none), Some(ButtonId::G6));
        assert_eq!(baseus.button_for_key(KVK_ANSI_Q, lock), Some(ButtonId::G7));
        assert_eq!(
            baseus.button_for_key(KVK_ANSI_Q, none),
            None,
            "a bare Q is typing"
        );
        assert_eq!(baseus.button_for_key(KVK_F11, lock), None);
        assert_eq!(
            baseus.layout.stock_action(ButtonId::G6),
            Some(Action::ShowDesktop)
        );
        assert!(!baseus.layout.needs_host_mode(ButtonId::G7));
    }

    #[test]
    fn g903_lightspeed_slots_match_the_hardware_dump() {
        let layout = GamingLayout::for_model_key("04087").unwrap();
        assert_eq!(layout, &G903_LIGHTSPEED);
        assert_eq!(layout.button_for_g_number(4), Some(ButtonId::Back));
        assert_eq!(layout.button_for_g_number(5), Some(ButtonId::Forward));
        assert_eq!(layout.button_for_g_number(6), Some(ButtonId::G6));
        assert_eq!(layout.button_for_g_number(9), Some(ButtonId::G9));
        assert_eq!(
            layout.button_for_g_number(10),
            Some(ButtonId::WheelTiltLeft)
        );
        assert_eq!(
            layout.button_for_g_number(11),
            Some(ButtonId::WheelTiltRight)
        );
        assert_eq!(layout.button_for_g_number(12), None);
        assert!(layout.needs_host_mode(ButtonId::G8));
        assert!(!layout.needs_host_mode(ButtonId::Forward));
    }

    #[test]
    fn g903_lightspeed_keeps_its_stock_functions() {
        let layout = &G903_LIGHTSPEED;
        assert_eq!(layout.stock_action(ButtonId::G6), Some(Action::MouseBack));
        assert_eq!(
            layout.stock_action(ButtonId::G7),
            Some(Action::MouseForward)
        );
        assert_eq!(
            layout.stock_action(ButtonId::G8),
            Some(Action::PreviousDpiPreset)
        );
        assert_eq!(
            layout.stock_action(ButtonId::G9),
            Some(Action::NextDpiPreset)
        );
        assert_eq!(layout.stock_action(ButtonId::WheelTiltLeft), None);
    }

    #[test]
    fn standard_buttons_are_what_the_os_hook_remaps() {
        let layout = &STANDARD_BUTTONS;
        assert_eq!(layout.button_for_g_number(3), Some(ButtonId::MiddleClick));
        assert_eq!(layout.button_for_g_number(4), Some(ButtonId::Back));
        assert_eq!(layout.button_for_g_number(5), Some(ButtonId::Forward));
        assert_eq!(layout.button_for_g_number(6), None, "G6+ is model-specific");
        assert!(layout.remappable().all(ButtonId::is_os_hook_button));
        assert!(!layout.remappable().any(|b| layout.needs_host_mode(b)));
    }

    #[test]
    fn unknown_models_have_no_layout() {
        assert_eq!(GamingLayout::for_model_key("04099"), None);
        assert_eq!(GamingLayout::for_model_key("2b042"), None);
    }

    #[test]
    fn g502_lightspeed_keeps_its_stock_functions() {
        let layout = &G502_LIGHTSPEED;
        assert_eq!(layout.stock_action(ButtonId::G6), Some(Action::DpiShift));
        assert_eq!(
            layout.stock_action(ButtonId::G7),
            Some(Action::PreviousDpiPreset)
        );
        assert_eq!(
            layout.stock_action(ButtonId::G8),
            Some(Action::NextDpiPreset)
        );
        assert_eq!(layout.stock_action(ButtonId::G9), Some(Action::None));
        assert_eq!(layout.stock_action(ButtonId::WheelTiltLeft), None);
        assert_eq!(layout.stock_action(ButtonId::Back), None);
        assert!(layout.needs_host_mode(ButtonId::WheelTiltRight));
    }

    #[test]
    fn the_default_of_a_g_button_comes_from_its_layout() {
        use super::super::defaults::{default_binding, default_binding_on};
        let g502 = Some(&G502_LIGHTSPEED);
        assert_eq!(default_binding_on(g502, ButtonId::G6), Action::DpiShift);
        assert_eq!(default_binding_on(None, ButtonId::G6), Action::None);
        assert_eq!(default_binding(ButtonId::G8), Action::None);
        for button in [
            ButtonId::Back,
            ButtonId::WheelTiltLeft,
            ButtonId::MiddleClick,
        ] {
            assert_eq!(default_binding_on(g502, button), default_binding(button));
        }
    }

    /// Gesture mode and wheel hold resolve defaults without a layout, which
    /// is right only while no G-series button offers them.
    #[test]
    fn g_buttons_offer_neither_gesture_mode_nor_wheel_hold() {
        for button in ButtonId::G_BUTTONS {
            assert!(!button.supports_gesture_mode(), "{button:?}");
            assert!(!button.supports_wheel_hold(), "{button:?}");
        }
    }

    #[test]
    fn every_g_button_on_a_dumped_layout_has_a_stock_function() {
        for layout in LAYOUTS {
            for slot in layout.slots {
                if ButtonId::G_BUTTONS.contains(&slot.button) {
                    assert!(slot.stock.is_some(), "{:?}", slot.button);
                }
            }
        }
    }

    #[test]
    fn a_stock_function_belongs_to_a_host_silent_slot() {
        for layout in LAYOUTS {
            for slot in layout.slots {
                if slot.stock.is_some() {
                    assert!(slot.host_silent, "{:?}", slot.button);
                }
            }
        }
    }
}
