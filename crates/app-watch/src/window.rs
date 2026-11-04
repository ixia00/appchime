pub fn has_main_window(pids: &[u32]) -> Option<bool> {
    imp::has_main_window(pids)
}

pub fn accessibility_trusted(prompt: bool) -> bool {
    imp::accessibility_trusted(prompt)
}

#[cfg(target_os = "macos")]
mod imp {
    use accessibility_sys::*;
    use core_foundation::array::{CFArray, CFArrayRef};
    use core_foundation::base::{CFRelease, CFType, CFTypeRef, TCFType};
    use core_foundation::boolean::CFBoolean;
    use core_foundation::dictionary::CFDictionary;
    use core_foundation::string::{CFString, CFStringRef};

    pub fn accessibility_trusted(prompt: bool) -> bool {
        unsafe {
            let key = CFString::wrap_under_get_rule(kAXTrustedCheckOptionPrompt);
            let value = if prompt { CFBoolean::true_value() } else { CFBoolean::false_value() };
            let options = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), value.as_CFType())]);
            AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef())
        }
    }

    pub fn has_main_window(pids: &[u32]) -> Option<bool> {
        if !accessibility_trusted(false) {
            return None;
        }
        Some(pids.iter().any(|&pid| unsafe { pid_has_standard_window(pid as i32) }))
    }

    unsafe fn pid_has_standard_window(pid: i32) -> bool {
        let app = AXUIElementCreateApplication(pid);
        if app.is_null() {
            return false;
        }
        let attr = CFString::new(kAXWindowsAttribute);
        let mut value: CFTypeRef = std::ptr::null();
        let err = AXUIElementCopyAttributeValue(app, attr.as_concrete_TypeRef(), &mut value);
        CFRelease(app as CFTypeRef);
        if err != kAXErrorSuccess || value.is_null() {
            return false;
        }
        let windows: CFArray<CFType> = CFArray::wrap_under_create_rule(value as CFArrayRef);
        let subrole_attr = CFString::new(kAXSubroleAttribute);
        windows.iter().any(|w| {
            let mut sub: CFTypeRef = std::ptr::null();
            let element = w.as_CFTypeRef() as AXUIElementRef;
            if AXUIElementCopyAttributeValue(element, subrole_attr.as_concrete_TypeRef(), &mut sub)
                != kAXErrorSuccess
                || sub.is_null()
            {
                return false;
            }
            CFString::wrap_under_create_rule(sub as CFStringRef).to_string() == kAXStandardWindowSubrole
        })
    }
}

#[cfg(target_os = "windows")]
mod imp {
    use windows::core::BOOL;
    use windows::Win32::Foundation::{HWND, LPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindow, GetWindowLongW, GetWindowThreadProcessId, IsWindowVisible, GWL_STYLE,
        GW_OWNER, WS_CAPTION,
    };

    pub fn accessibility_trusted(_prompt: bool) -> bool {
        true
    }

    struct Ctx<'a> {
        pids: &'a [u32],
        found: bool,
    }

    unsafe extern "system" fn visit(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let ctx = &mut *(lparam.0 as *mut Ctx);
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if !ctx.pids.contains(&pid) || !IsWindowVisible(hwnd).as_bool() {
            return BOOL(1);
        }
        let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
        let has_caption = style & WS_CAPTION.0 == WS_CAPTION.0;
        let is_top_level = GetWindow(hwnd, GW_OWNER).map(|o| o.is_invalid()).unwrap_or(true);
        if has_caption && is_top_level {
            ctx.found = true;
            return BOOL(0); 
        }
        BOOL(1)
    }

    pub fn has_main_window(pids: &[u32]) -> Option<bool> {
        let mut ctx = Ctx { pids, found: false };
        unsafe {
            let _ = EnumWindows(Some(visit), LPARAM(&mut ctx as *mut Ctx as isize));
        }
        Some(ctx.found)
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod imp {
    pub fn accessibility_trusted(_prompt: bool) -> bool {
        true
    }
    pub fn has_main_window(_pids: &[u32]) -> Option<bool> {
        None
    }
}