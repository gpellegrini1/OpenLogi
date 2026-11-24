//! Non-Logitech mice remapped through the OS input hook.
//!
//! These devices speak no HID++ and no vendor protocol OpenLogi models. They
//! are recognized only by their HID mouse collection (Generic Desktop `0x01`,
//! usage Mouse `0x02`) so the agent can list them and remap their standard
//! Middle/Back/Forward buttons by event-source identity. Nothing is ever read
//! from or written to them.

use crate::LOGITECH_VENDOR_ID;

/// Stable driver-family identifier carried by standalone generic-mouse inventory.
pub const GENERIC_MOUSE_DRIVER_ID: &str = "generic-mouse";
/// HID Generic Desktop usage page.
pub const GENERIC_DESKTOP_USAGE_PAGE: u16 = 0x0001;
/// HID Generic Desktop Mouse usage.
pub const MOUSE_USAGE_ID: u16 = 0x0002;
/// Apple's vendor ID: built-in trackpads and the Magic Mouse/Trackpad family,
/// whose multitouch surfaces the hook must never remap.
const APPLE_VENDOR_ID: u16 = 0x05ac;

/// Whether an HID node is a third-party mouse collection OpenLogi may list as
/// a generic mouse.
///
/// Logitech nodes belong to the HID++ layer, Apple nodes are trackpad-family
/// surfaces, a zero vendor is a virtual or built-in device, and a node named
/// like a trackpad is a pointing surface rather than a mouse.
#[must_use]
pub fn matches_generic_mouse(
    vendor_id: u16,
    usage_page: u16,
    usage_id: u16,
    product_name: &str,
) -> bool {
    is_generic_mouse_collection(vendor_id, usage_page, usage_id)
        && !matches!(vendor_id, 0 | APPLE_VENDOR_ID)
        && !is_trackpad_name(product_name)
}

/// Whether a raw-HID route's identity tuple names a generic-mouse collection.
///
/// No other standalone driver family claims a non-Logitech mouse collection,
/// so this tells a listed generic mouse apart from every other raw device
/// without carrying its driver id along.
#[must_use]
pub fn is_generic_mouse_collection(vendor_id: u16, usage_page: u16, usage_id: u16) -> bool {
    (usage_page, usage_id) == (GENERIC_DESKTOP_USAGE_PAGE, MOUSE_USAGE_ID)
        && vendor_id != LOGITECH_VENDOR_ID
}

/// Whether a product name reads like a touch surface rather than a mouse.
fn is_trackpad_name(name: &str) -> bool {
    // `no_std`: an ASCII case-insensitive substring search without allocating.
    let name = name.as_bytes();
    [&b"trackpad"[..], b"touchpad", b"touch pad"]
        .iter()
        .any(|needle| {
            name.windows(needle.len())
                .any(|window| window.eq_ignore_ascii_case(needle))
        })
}

#[cfg(test)]
mod tests {
    use super::matches_generic_mouse;
    use crate::LOGITECH_VENDOR_ID;

    #[test]
    fn third_party_mouse_collection_matches() {
        assert!(matches_generic_mouse(
            0xfff1,
            0x0001,
            0x0002,
            "Baseus Editor Pro"
        ));
    }

    #[test]
    fn sibling_collections_of_the_same_mouse_do_not_match() {
        // Keyboard, consumer, and vendor collections of the BS-007Pro.
        assert!(!matches_generic_mouse(
            0xfff1,
            0x0001,
            0x0006,
            "Baseus Editor Pro"
        ));
        assert!(!matches_generic_mouse(
            0xfff1,
            0x000c,
            0x0001,
            "Baseus Editor Pro"
        ));
        assert!(!matches_generic_mouse(
            0xfff1,
            0xfff0,
            0x0001,
            "Baseus Editor Pro"
        ));
    }

    #[test]
    fn logitech_apple_virtual_and_trackpads_are_excluded() {
        assert!(!matches_generic_mouse(
            LOGITECH_VENDOR_ID,
            0x0001,
            0x0002,
            "MX Master 3S"
        ));
        assert!(!matches_generic_mouse(
            0x05ac,
            0x0001,
            0x0002,
            "Magic Mouse"
        ));
        assert!(!matches_generic_mouse(
            0,
            0x0001,
            0x0002,
            "Apple Internal Keyboard / Trackpad"
        ));
        assert!(!matches_generic_mouse(
            0x1234,
            0x0001,
            0x0002,
            "Some Touchpad"
        ));
    }
}
