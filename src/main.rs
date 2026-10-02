#![windows_subsystem = "windows"]
#![allow(dead_code, unused_variables, unused_imports)]

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicU32, AtomicU64, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

// Win32 Types
#[repr(C)]
struct POINT {
    x: i32,
    y: i32,
}

#[repr(C)]
struct RECT {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[repr(C)]
struct PAINTSTRUCT {
    hdc: isize,
    f_erase: i32,
    rc_paint: RECT,
    f_restore: i32,
    f_inc_update: i32,
    rgb_reserved: [u8; 32],
}

#[repr(C)]
struct BITMAPINFOHEADER {
    bi_size: u32,
    bi_width: i32,
    bi_height: i32,
    bi_planes: u16,
    bi_bit_count: u16,
    bi_compression: u32,
    bi_size_image: u32,
    bi_x_pels_per_meter: i32,
    bi_y_pels_per_meter: i32,
    bi_clr_used: u32,
    bi_clr_important: u32,
}

#[repr(C)]
struct BITMAPINFO {
    bmi_header: BITMAPINFOHEADER,
    bmi_colors: [u32; 1],
}

#[repr(C)]
struct BLENDFUNCTION {
    blend_op: u8,
    blend_flags: u8,
    source_constant_alpha: u8,
    alpha_format: u8,
}
const AC_SRC_OVER: u8 = 0x00;
const AC_SRC_ALPHA: u8 = 0x01;
const ULW_ALPHA: u32 = 0x00000002;

#[repr(C)]
struct ICONINFO {
    f_icon: i32,
    x_hotspot: u32,
    y_hotspot: u32,
    hbm_mask: isize,
    hbm_color: isize,
}

#[repr(C)]
struct MSG {
    hwnd: isize,
    message: u32,
    w_param: usize,
    l_param: isize,
    time: u32,
    pt: POINT,
}

#[repr(C)]
struct KBDLLHOOKSTRUCT {
    vk_code: u32,
    scan_code: u32,
    flags: u32,
    time: u32,
    extra_info: usize,
}

#[repr(C)]
struct WNDCLASSEXW {
    cb_size: u32,
    style: u32,
    lpfn_wnd_proc: WNDPROC,
    cb_cls_extra: i32,
    cb_wnd_extra: i32,
    h_instance: isize,
    h_icon: isize,
    h_cursor: isize,
    h_br_background: isize,
    lpsz_menu_name: *const u16,
    lpsz_class_name: *const u16,
    h_icon_sm: isize,
}

#[repr(C)]
struct NOTIFYICONDATAW {
    cb_size: u32,
    h_wnd: isize,
    u_id: u32,
    u_flags: u32,
    u_callback_message: u32,
    h_icon: isize,
    sz_tip: [u16; 128],
    dw_state: u32,
    dw_state_mask: u32,
    sz_info: [u16; 256],
    u_timeout_or_version: u32,
    sz_info_title: [u16; 64],
    dw_info_flags: u32,
    guid_item: [u8; 16],
    h_balloon_icon: isize,
}

#[repr(C)]
struct STARTUPINFOW {
    cb: u32,
    lp_reserved: *mut u16,
    lp_desktop: *mut u16,
    lp_title: *mut u16,
    dw_x: u32,
    dw_y: u32,
    dw_x_size: u32,
    dw_y_size: u32,
    dw_x_count_chars: u32,
    dw_y_count_chars: u32,
    dw_fill_attribute: u32,
    dw_flags: u32,
    w_show_window: u16,
    cb_reserved2: u16,
    lp_reserved2: *mut u8,
    h_std_input: isize,
    h_std_output: isize,
    h_std_error: isize,
}

#[repr(C)]
struct PROCESS_INFORMATION {
    h_process: isize,
    h_thread: isize,
    dw_process_id: u32,
    dw_thread_id: u32,
}

type HOOKPROC = unsafe extern "system" fn(code: i32, w_param: usize, l_param: isize) -> isize;
type WNDPROC = unsafe extern "system" fn(hwnd: isize, msg: u32, w_param: usize, l_param: isize) -> isize;
type WNDENUMPROC = unsafe extern "system" fn(hwnd: isize, lparam: isize) -> i32;

// Win32 Constants
const WH_KEYBOARD_LL: i32 = 13;
const WM_CREATE: u32 = 0x0001;
const WM_DESTROY: u32 = 0x0002;
const WM_CLOSE: u32 = 0x0010;
const WM_COMMAND: u32 = 0x0111;
const WM_KEYDOWN: usize = 0x0100;
const WM_KEYUP: usize = 0x0101;
const WM_SYSKEYDOWN: usize = 0x0104;
const WM_SYSKEYUP: usize = 0x0105;
const WM_LBUTTONDOWN: usize = 0x0201;
const WM_LBUTTONUP: usize = 0x0202;
const WM_LBUTTONDBLCLK: usize = 0x0203;
const WM_RBUTTONUP: usize = 0x0205;
const WM_CONTEXTMENU: usize = 0x007B;
const NIN_SELECT: usize = 0x0400;
const NIN_KEYSELECT: usize = 0x0401;

const CS_DBLCLKS: u32 = 0x0008;
const SW_HIDE: i32 = 0;
const SW_SHOWNOACTIVATE: i32 = 4;
const SW_RESTORE: i32 = 9;

const GWL_STYLE: i32 = -16;
const WS_POPUP: u32 = 0x80000000;
const WS_THICKFRAME: u32 = 0x00040000;
const WS_MAXIMIZEBOX: u32 = 0x00010000;
const WS_EX_TOPMOST: u32 = 0x00000008;
const WS_EX_TOOLWINDOW: u32 = 0x00000080;
const WS_EX_LAYERED: u32 = 0x00080000;
const WS_EX_NOACTIVATE: u32 = 0x08000000;
const WS_EX_TRANSPARENT: u32 = 0x00000020;
const LWA_ALPHA: u32 = 0x00000002;

const HWND_TOPMOST: isize = -1;
const SWP_NOSIZE: u32 = 0x0001;
const SWP_NOMOVE: u32 = 0x0002;
const SWP_NOZORDER: u32 = 0x0004;
const SWP_NOACTIVATE: u32 = 0x0010;
const SWP_FRAMECHANGED: u32 = 0x0020;
const SWP_SHOWWINDOW: u32 = 0x0040;

const WM_PAINT: u32 = 0x000F;
const WM_TIMER: u32 = 0x0113;
const WM_USER: u32 = 0x0400;
const WM_TRAYICON: u32 = WM_USER + 100;
const WM_UPDATE_TRAY: u32 = WM_USER + 101;
const WM_SHOW_TOAST: u32 = WM_USER + 201;
const WM_SETICON: u32 = 0x0080;
const ICON_SMALL: usize = 0;
const ICON_BIG: usize = 1;

const DT_SINGLELINE: u32 = 0x0020;
const DT_NOPREFIX: u32 = 0x0800;
const DT_VCENTER: u32 = 0x0004;

const MOUSEEVENTF_MOVE: u32 = 0x0001;
const MOUSEEVENTF_LEFTDOWN: u32 = 0x0002;
const MOUSEEVENTF_LEFTUP: u32 = 0x0004;
const MOUSEEVENTF_RIGHTDOWN: u32 = 0x0008;
const MOUSEEVENTF_RIGHTUP: u32 = 0x0010;
const MOUSEEVENTF_WHEEL: u32 = 0x0800;
const WHEEL_DELTA: i32 = 120;

// Virtual Key Constants
const VK_SPACE: u32 = 0x20;
const VK_ESCAPE: u32 = 0x1B;
const VK_F8: u32 = 0x77;
const VK_SHIFT: u32 = 0x10;
const VK_CONTROL: u32 = 0x11;
const VK_MENU: u32 = 0x12; // Alt
const VK_LSHIFT: u32 = 0xA0;
const VK_RSHIFT: u32 = 0xA1;
const VK_LCONTROL: u32 = 0xA2;
const VK_RCONTROL: u32 = 0xA3;
const VK_LMENU: u32 = 0xA4;
const VK_RMENU: u32 = 0xA5;

// Tray & Menu IDs
const ID_TRAY_ICON: u32 = 1001;
const IDM_TOGGLE: usize = 2001;
const IDM_OPEN_UI: usize = 2002;
const IDM_EXIT: usize = 2003;

const NIF_MESSAGE: u32 = 0x01;
const NIF_ICON: u32 = 0x02;
const NIF_TIP: u32 = 0x04;
const NIM_ADD: u32 = 0;
const NIM_MODIFY: u32 = 1;
const NIM_DELETE: u32 = 2;

const MF_STRING: u32 = 0x00000000;
const MF_SEPARATOR: u32 = 0x00000800;
const TPM_RIGHTBUTTON: u32 = 0x0002;

#[link(name = "user32")]
unsafe extern "system" {
    fn SetWindowsHookExW(id_hook: i32, lpfn: HOOKPROC, hmod: isize, dw_thread_id: u32) -> isize;
    fn UnhookWindowsHookEx(hhk: isize) -> i32;
    fn CallNextHookEx(hhk: isize, n_code: i32, w_param: usize, l_param: isize) -> isize;
    fn GetMessageW(lp_msg: *mut MSG, h_wnd: isize, w_msg_filter_min: u32, w_msg_filter_max: u32) -> i32;
    fn TranslateMessage(lp_msg: *const MSG) -> i32;
    fn DispatchMessageW(lp_msg: *const MSG) -> isize;
    fn PostQuitMessage(n_exit_code: i32);
    fn GetCursorPos(lp_point: *mut POINT) -> i32;
    fn SetCursorPos(x: i32, y: i32) -> i32;
    fn mouse_event(dw_flags: u32, dx: u32, dy: u32, dw_data: u32, dw_extra_info: usize);
    fn OpenDesktopW(lpsz_desktop: *const u16, dw_flags: u32, f_inherit: i32, dw_desired_access: u32) -> isize;
    fn SetThreadDesktop(h_desktop: isize) -> i32;
    fn CloseDesktop(h_desktop: isize) -> i32;
    fn EnumDesktopWindows(h_desktop: isize, lpfn: WNDENUMPROC, lparam: isize) -> i32;
    fn GetWindowTextW(hwnd: isize, lp_string: *mut u16, n_max_count: i32) -> i32;
    fn ShowWindow(hwnd: isize, n_cmd_show: i32) -> i32;
    fn BringWindowToTop(hwnd: isize) -> i32;
    fn GetWindowLongW(hwnd: isize, n_index: i32) -> i32;
    fn SetWindowLongW(hwnd: isize, n_index: i32, dw_new_long: i32) -> i32;
    fn SetWindowPos(hwnd: isize, hwnd_insert_after: isize, x: i32, y: i32, cx: i32, cy: i32, u_flags: u32) -> i32;
    fn RegisterClassExW(lp_wnd_class: *const WNDCLASSEXW) -> u16;
    fn CreateWindowExW(dw_ex_style: u32, lp_class_name: *const u16, lp_window_name: *const u16, dw_style: u32, x: i32, y: i32, n_width: i32, n_height: i32, h_wnd_parent: isize, h_menu: isize, h_instance: isize, lp_param: *mut std::ffi::c_void) -> isize;
    fn DefWindowProcW(h_wnd: isize, msg: u32, w_param: usize, l_param: isize) -> isize;
    fn DestroyWindow(h_wnd: isize) -> i32;
    fn GetWindowThreadProcessId(hwnd: isize, lpdw_process_id: *mut u32) -> u32;
    fn IsWindow(hwnd: isize) -> i32;
    fn LoadCursorW(h_instance: isize, lp_cursor_name: *const u16) -> isize;
    fn LoadIconW(h_instance: isize, lp_icon_name: *const u16) -> isize;
    fn CreateIconIndirect(piconinfo: *const ICONINFO) -> isize;
    fn DestroyIcon(hicon: isize) -> i32;
    fn CreatePopupMenu() -> isize;
    fn AppendMenuW(h_menu: isize, u_flags: u32, u_id_new_item: usize, lp_new_item: *const u16) -> i32;
    fn TrackPopupMenu(h_menu: isize, u_flags: u32, x: i32, y: i32, n_reserved: i32, h_wnd: isize, prc_rect: *const std::ffi::c_void) -> i32;
    fn DestroyMenu(h_menu: isize) -> i32;
    fn SetForegroundWindow(h_wnd: isize) -> i32;
    fn BeginPaint(hwnd: isize, lp_paint: *mut PAINTSTRUCT) -> isize;
    fn EndPaint(hwnd: isize, lp_paint: *const PAINTSTRUCT) -> i32;
    fn InvalidateRect(hwnd: isize, lp_rect: *const RECT, b_erase: i32) -> i32;
    fn SetWindowRgn(hwnd: isize, hrgn: isize, b_redraw: i32) -> i32;
    fn SetLayeredWindowAttributes(hwnd: isize, cr_key: u32, b_alpha: u8, dw_flags: u32) -> i32;
    fn SetTimer(hwnd: isize, n_id_event: usize, u_elapse: u32, lp_timer_func: isize) -> usize;
    fn KillTimer(hwnd: isize, u_id_event: usize) -> i32;
    fn GetSystemMetrics(n_index: i32) -> i32;
    fn DrawTextW(hdc: isize, lpch_text: *const u16, cch_text: i32, lprc: *mut RECT, format: u32) -> i32;
    fn PostMessageW(hwnd: isize, msg: u32, w_param: usize, l_param: isize) -> i32;
    fn SendMessageW(hwnd: isize, msg: u32, w_param: usize, l_param: isize) -> isize;
    fn GetDC(hwnd: isize) -> isize;
    fn ReleaseDC(hwnd: isize, hdc: isize) -> i32;
    fn UpdateLayeredWindow(
        hwnd: isize,
        hdc_dst: isize,
        ppt_dst: *const POINT,
        psize: *const POINT,
        hdc_src: isize,
        ppt_src: *const POINT,
        cr_key: u32,
        pblend: *const BLENDFUNCTION,
        dw_flags: u32,
    ) -> i32;
}

#[link(name = "gdi32")]
unsafe extern "system" {
    fn CreateSolidBrush(color: u32) -> isize;
    fn CreatePen(i_style: i32, c_width: i32, color: u32) -> isize;
    fn SelectObject(hdc: isize, h: isize) -> isize;
    fn DeleteObject(ho: isize) -> i32;
    fn RoundRect(hdc: isize, left: i32, top: i32, right: i32, bottom: i32, width: i32, height: i32) -> i32;
    fn Ellipse(hdc: isize, left: i32, top: i32, right: i32, bottom: i32) -> i32;
    fn SetBkMode(hdc: isize, mode: i32) -> i32;
    fn SetTextColor(hdc: isize, color: u32) -> u32;
    fn CreateRoundRectRgn(x1: i32, y1: i32, x2: i32, y2: i32, w: i32, h: i32) -> isize;
    fn CreateFontW(
        c_height: i32, c_width: i32, c_escapement: i32, c_orientation: i32,
        c_weight: i32, b_italic: u32, b_underline: u32, b_strike_out: u32,
        i_char_set: u32, i_out_precision: u32, i_clip_precision: u32,
        i_quality: u32, i_pitch_and_family: u32, psz_face_name: *const u16,
    ) -> isize;
    fn CreateCompatibleDC(hdc: isize) -> isize;
    fn DeleteDC(hdc: isize) -> i32;
    fn CreateDIBSection(
        hdc: isize,
        pbmi: *const BITMAPINFO,
        usage: u32,
        ppv_bits: *mut *mut u8,
        h_section: isize,
        offset: u32,
    ) -> isize;
    fn CreateBitmap(n_width: i32, n_height: i32, n_planes: u32, n_bit_count: u32, lp_bits: *const u8) -> isize;
}

#[link(name = "shell32")]
unsafe extern "system" {
    fn Shell_NotifyIconW(dw_message: u32, lp_data: *const NOTIFYICONDATAW) -> i32;
    fn IsUserAnAdmin() -> i32;
    fn ShellExecuteW(
        hwnd: isize,
        lp_operation: *const u16,
        lp_file: *const u16,
        lp_parameters: *const u16,
        lp_directory: *const u16,
        n_show_cmd: i32,
    ) -> isize;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn Beep(dw_freq: u32, dw_duration: u32) -> i32;
    fn GetModuleHandleW(lp_module_name: *const u16) -> isize;
    fn CreateProcessW(
        lp_application_name: *const u16,
        lp_command_line: *mut u16,
        lp_process_attributes: *mut std::ffi::c_void,
        lp_thread_attributes: *mut std::ffi::c_void,
        b_inherit_handles: i32,
        dw_creation_flags: u32,
        lp_environment: *mut std::ffi::c_void,
        lp_current_directory: *const u16,
        lp_startup_info: *const STARTUPINFOW,
        lp_process_information: *mut PROCESS_INFORMATION,
    ) -> i32;
    fn CloseHandle(h_object: isize) -> i32;
}

// Embedded Fluent UI HTML & Icons
const UI_HTML: &str = include_str!("ui.html");
const TRAY_ON_16_RAW: &[u8; 1024] = include_bytes!("tray_on_16.raw");
const TRAY_OFF_16_RAW: &[u8; 1024] = include_bytes!("tray_off_16.raw");
const TRAY_ON_32_RAW: &[u8; 4096] = include_bytes!("tray_on_32.raw");
const TRAY_OFF_32_RAW: &[u8; 4096] = include_bytes!("tray_off_32.raw");
const APP_ICON_48_RAW: &[u8; 9216] = include_bytes!("app_icon_48.raw");
const ICON_256_PNG: &[u8] = include_bytes!("icon-256.png");
const ICON_64_PNG: &[u8] = include_bytes!("icon-64.png");
const FAVICON_ICO: &[u8] = include_bytes!("favicon.ico");

// Global Atomic State
static HOOK_HANDLE: AtomicIsize = AtomicIsize::new(0);
static MAIN_HWND: AtomicIsize = AtomicIsize::new(0);
static MOUSE_MODE: AtomicBool = AtomicBool::new(true);
static TRAY_ICON_ON: AtomicIsize = AtomicIsize::new(0);
static TRAY_ICON_OFF: AtomicIsize = AtomicIsize::new(0);
static APP_ICON_BIG: AtomicIsize = AtomicIsize::new(0);

// Toast HUD State
static TOAST_HWND: AtomicIsize = AtomicIsize::new(0);
static TOAST_HIDE_TIME: AtomicU64 = AtomicU64::new(0);
static TOAST_IS_ON: AtomicBool = AtomicBool::new(false);
static TOAST_POS: Mutex<String> = Mutex::new(String::new());

fn get_toast_pos() -> String {
    let lock = TOAST_POS.lock().unwrap();
    if lock.is_empty() {
        "top-center".to_string()
    } else {
        lock.clone()
    }
}

fn set_toast_pos(pos: &str) {
    let mut lock = TOAST_POS.lock().unwrap();
    *lock = pos.to_string();
}

static LANG: Mutex<String> = Mutex::new(String::new());

fn get_lang() -> String {
    let lock = LANG.lock().unwrap();
    if lock.is_empty() {
        "vi".to_string()
    } else {
        lock.clone()
    }
}

fn set_lang(l: &str) {
    let mut lock = LANG.lock().unwrap();
    if l == "en" {
        *lock = "en".to_string();
    } else {
        *lock = "vi".to_string();
    }
}

fn calc_toast_pos(pos: &str) -> (i32, i32) {
    unsafe {
        let sw = GetSystemMetrics(0);
        let sh = GetSystemMetrics(1);
        let tw = 290;
        let th = 44;
        let pad = 24;
        match pos {
            "top-left" => (pad, pad),
            "top-right" => (sw - tw - pad, pad),
            "bottom-center" => ((sw - tw) / 2, sh - th - 54),
            "bottom-right" => (sw - tw - pad, sh - th - 54),
            "bottom-left" => (pad, sh - th - 54),
            _ => ((sw - tw) / 2, pad),
        }
    }
}

// Customizable Parameters
static MIN_SPEED_X10: AtomicU32 = AtomicU32::new(7);   // 0.7 px
static MAX_SPEED_X10: AtomicU32 = AtomicU32::new(75);  // 7.5 px
static ACCEL_TICKS: AtomicU32 = AtomicU32::new(56);    // ~450ms
static TURBO_MULT_X10: AtomicU32 = AtomicU32::new(20); // 2.0x (Shift)
static ENABLE_BEEP: AtomicBool = AtomicBool::new(true);

// Dynamic Hotkey Rules
#[derive(Clone, Debug)]
struct HotkeyRule {
    combo: String,
    enabled: bool,
    action: String,
    req_ctrl: bool,
    req_shift: bool,
    req_alt: bool,
    trigger_vk: u32,
    is_chord_shift_alt: bool,
    is_chord_alt_space: bool,
    is_chord_ctrl_shift: bool,
    is_chord_ctrl_alt: bool,
    is_exit: bool,
}

static HOTKEY_RULES: Mutex<Vec<HotkeyRule>> = Mutex::new(Vec::new());

fn parse_hotkey(combo: &str, enabled: bool) -> HotkeyRule {
    let upper = combo.to_uppercase();
    let parts: Vec<&str> = upper.split('+').map(|s| s.trim()).collect();
    let has_ctrl = parts.iter().any(|&p| p == "CTRL" || p == "CONTROL");
    let has_shift = parts.iter().any(|&p| p == "SHIFT");
    let has_alt = parts.iter().any(|&p| p == "ALT");
    let has_space = parts.iter().any(|&p| p == "SPACE");
    let has_esc = parts.iter().any(|&p| p == "ESC" || p == "ESCAPE");

    let is_exit = has_esc;

    let is_chord_shift_alt = parts.len() == 2 && has_shift && has_alt;
    let is_chord_alt_space = parts.len() == 2 && has_alt && has_space;
    let is_chord_ctrl_shift = parts.len() == 2 && has_ctrl && has_shift;
    let is_chord_ctrl_alt = parts.len() == 2 && has_ctrl && has_alt;

    let mut trigger_vk = 0;
    if !is_chord_shift_alt && !is_chord_alt_space && !is_chord_ctrl_shift && !is_chord_ctrl_alt {
        for &p in &parts {
            if p != "CTRL" && p != "CONTROL" && p != "SHIFT" && p != "ALT" && p != "WIN" {
                trigger_vk = name_to_vk(p);
                break;
            }
        }
    }

    HotkeyRule {
        combo: combo.to_string(),
        enabled,
        action: if is_exit { "exit".to_string() } else { "toggle".to_string() },
        req_ctrl: has_ctrl,
        req_shift: has_shift,
        req_alt: has_alt,
        trigger_vk,
        is_chord_shift_alt,
        is_chord_alt_space,
        is_chord_ctrl_shift,
        is_chord_ctrl_alt,
        is_exit,
    }
}

fn parse_hotkeys_json(body: &str) -> Option<Vec<HotkeyRule>> {
    let start_idx = body.find("\"hotkeys\"")?;
    let slice = &body[start_idx..];
    let open_bracket = slice.find('[')?;
    let close_bracket = slice.find(']')?;
    let arr_str = &slice[open_bracket + 1..close_bracket];

    let mut rules = Vec::new();
    for item in arr_str.split('}') {
        if let Some(combo_start) = item.find("\"combo\"") {
            let combo_slice = &item[combo_start..];
            if let Some(first_colon) = combo_slice.find(':') {
                let rest = &combo_slice[first_colon + 1..];
                if let Some(q1) = rest.find('"') {
                    let rest_q = &rest[q1 + 1..];
                    if let Some(q2) = rest_q.find('"') {
                        let combo = &rest_q[..q2];
                        let enabled = if let Some(e_idx) = item.find("\"enabled\"") {
                            let e_slice = &item[e_idx..];
                            !e_slice.contains("false")
                        } else {
                            true
                        };
                        rules.push(parse_hotkey(combo, enabled));
                    }
                }
            }
        }
    }
    Some(rules)
}

fn init_default_hotkeys() {
    let mut lock = HOTKEY_RULES.lock().unwrap();
    if lock.is_empty() {
        lock.push(parse_hotkey("Shift + Alt", true));
        lock.push(parse_hotkey("F8", true));
        lock.push(parse_hotkey("Alt + Space", false));
        lock.push(parse_hotkey("Esc", true));
    }
}

// Keybindings (Virtual Keys)
static VK_BIND_UP: AtomicU32 = AtomicU32::new(0x57);     // W
static VK_BIND_DOWN: AtomicU32 = AtomicU32::new(0x53);   // S
static VK_BIND_LEFT: AtomicU32 = AtomicU32::new(0x41);   // A
static VK_BIND_RIGHT: AtomicU32 = AtomicU32::new(0x44);  // D
static VK_BIND_LCLICK: AtomicU32 = AtomicU32::new(0x20); // Space
static VK_BIND_RCLICK: AtomicU32 = AtomicU32::new(0x58); // X
static VK_BIND_SCROLL_UP: AtomicU32 = AtomicU32::new(0x45);   // E
static VK_BIND_SCROLL_DOWN: AtomicU32 = AtomicU32::new(0x51); // Q

// Movement Flags
static MOVE_UP: AtomicBool = AtomicBool::new(false);
static MOVE_DOWN: AtomicBool = AtomicBool::new(false);
static MOVE_LEFT: AtomicBool = AtomicBool::new(false);
static MOVE_RIGHT: AtomicBool = AtomicBool::new(false);

// Physical Modifier States
static SHIFT_HELD: AtomicBool = AtomicBool::new(false);
static CTRL_HELD: AtomicBool = AtomicBool::new(false);
static ALT_HELD: AtomicBool = AtomicBool::new(false);

// Scroll Flags
static SCROLL_UP: AtomicBool = AtomicBool::new(false);
static SCROLL_DOWN: AtomicBool = AtomicBool::new(false);

// Mouse Click State
static LEFT_HELD: AtomicBool = AtomicBool::new(false);
static RIGHT_HELD: AtomicBool = AtomicBool::new(false);

fn to_wstring(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn vk_to_name(vk: u32) -> String {
    match vk {
        0x20 => "Space".to_string(),
        0x0D => "Enter".to_string(),
        0x1B => "Esc".to_string(),
        0x09 => "Tab".to_string(),
        0x14 => "Caps Lock".to_string(),
        0x10 | 0xA0 | 0xA1 => "Shift".to_string(),
        0x11 | 0xA2 | 0xA3 => "Ctrl".to_string(),
        0x12 | 0xA4 | 0xA5 => "Alt".to_string(),
        0x25 => "Left".to_string(),
        0x26 => "Up".to_string(),
        0x27 => "Right".to_string(),
        0x28 => "Down".to_string(),
        0x70..=0x87 => format!("F{}", vk - 0x70 + 1),
        0x30..=0x39 => format!("{}", (vk as u8) as char),
        0x41..=0x5A => format!("{}", (vk as u8) as char),
        _ => format!("Key {:#X}", vk),
    }
}

fn name_to_vk(name: &str) -> u32 {
    let upper = name.trim().to_uppercase();
    match upper.as_str() {
        "SPACE" => 0x20,
        "ENTER" => 0x0D,
        "ESC" | "ESCAPE" => 0x1B,
        "TAB" => 0x09,
        "CAPS LOCK" | "CAPSLOCK" => 0x14,
        "SHIFT" => 0x10,
        "CTRL" | "CONTROL" => 0x11,
        "ALT" => 0x12,
        "UP" | "ARROWUP" => 0x26,
        "DOWN" | "ARROWDOWN" => 0x28,
        "LEFT" | "ARROWLEFT" => 0x25,
        "RIGHT" | "ARROWRIGHT" => 0x27,
        "F1" => 0x70, "F2" => 0x71, "F3" => 0x72, "F4" => 0x73,
        "F5" => 0x74, "F6" => 0x75, "F7" => 0x76, "F8" => 0x77,
        "F9" => 0x78, "F10" => 0x79, "F11" => 0x7A, "F12" => 0x7B,
        s if s.len() == 1 => {
            let ch = s.chars().next().unwrap();
            if ch.is_ascii_alphanumeric() {
                ch as u32
            } else {
                0x57
            }
        }
        _ => 0x57,
    }
}

fn attach_to_default_desktop() {
    unsafe {
        let name = to_wstring("Default");
        let hdesk = OpenDesktopW(name.as_ptr(), 0, 0, 0x01FF);
        if hdesk != 0 {
            SetThreadDesktop(hdesk);
        }
    }
}

fn trigger_toast(is_on: bool) {
    let hwnd = TOAST_HWND.load(Ordering::SeqCst);
    if hwnd != 0 {
        unsafe {
            PostMessageW(hwnd, WM_SHOW_TOAST, if is_on { 1 } else { 0 }, 0);
        }
    }
}

fn update_tray_icon(is_on: bool) {
    let hwnd = MAIN_HWND.load(Ordering::SeqCst);
    if hwnd != 0 {
        unsafe {
            PostMessageW(hwnd, WM_UPDATE_TRAY, if is_on { 1 } else { 0 }, 0);
        }
    }
}

fn set_mouse_mode(enabled: bool) {
    let prev = MOUSE_MODE.swap(enabled, Ordering::SeqCst);
    if prev != enabled {
        trigger_toast(enabled);
        update_tray_icon(enabled);
        if enabled {
            if ENABLE_BEEP.load(Ordering::Relaxed) {
                thread::spawn(|| unsafe { Beep(880, 80) });
            }
        } else {
            reset_all_keys();
            if ENABLE_BEEP.load(Ordering::Relaxed) {
                thread::spawn(|| unsafe { Beep(440, 80) });
            }
        }
    }
}

fn reset_all_keys() {
    MOVE_UP.store(false, Ordering::Relaxed);
    MOVE_DOWN.store(false, Ordering::Relaxed);
    MOVE_LEFT.store(false, Ordering::Relaxed);
    MOVE_RIGHT.store(false, Ordering::Relaxed);
    SCROLL_UP.store(false, Ordering::Relaxed);
    SCROLL_DOWN.store(false, Ordering::Relaxed);

    if LEFT_HELD.swap(false, Ordering::Relaxed) {
        unsafe { mouse_event(MOUSEEVENTF_LEFTUP, 0, 0, 0, 0) };
    }
    if RIGHT_HELD.swap(false, Ordering::Relaxed) {
        unsafe { mouse_event(MOUSEEVENTF_RIGHTUP, 0, 0, 0, 0) };
    }
}

static UI_HWND: AtomicIsize = AtomicIsize::new(0);
static SPAWNED_PID: AtomicU32 = AtomicU32::new(0);
static FOUND_HWND: AtomicIsize = AtomicIsize::new(0);
static LAST_SPAWN_TIME: AtomicU64 = AtomicU64::new(0);

unsafe extern "system" fn enum_desktop_windows_proc(hwnd: isize, lparam: isize) -> i32 {
    unsafe {
        let target_pid = lparam as u32;
        let mut proc_id = 0u32;
        GetWindowThreadProcessId(hwnd, &mut proc_id);

        // STRICT MATCH: Only inspect windows created by OUR exact spawned Edge process!
        // Never touch the user's personal browser or other application windows!
        if target_pid != 0 && proc_id != target_pid {
            return 1;
        }

        let mut buf = [0u16; 512];
        let len = GetWindowTextW(hwnd, buf.as_mut_ptr(), 512);
        if len > 0 {
            let title = String::from_utf16_lossy(&buf[..len as usize]);
            if !title.contains("KeyMouseHost") && !title.contains("Toast") && !title.contains("KM_") {
                FOUND_HWND.store(hwnd, Ordering::SeqCst);
                return 0; // stop enumeration
            }
        }
        1
    }
}

fn open_fluent_ui() {
    unsafe {
        // 1. If existing UI window is already open and valid, restore and focus it directly
        let existing = UI_HWND.load(Ordering::SeqCst);
        if existing != 0 && IsWindow(existing) != 0 {
            ShowWindow(existing, SW_RESTORE);
            SetForegroundWindow(existing);
            BringWindowToTop(existing);
            return;
        }

        // 2. Debounce launches (avoid double launching when double clicking)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let last = LAST_SPAWN_TIME.load(Ordering::Relaxed);
        if now.saturating_sub(last) < 1200 {
            return;
        }
        LAST_SPAWN_TIME.store(now, Ordering::Relaxed);

        // 3. Isolated Edge / Chrome user profile dynamically resolved for any user account
        let profile_dir = if let Ok(appdata) = std::env::var("LOCALAPPDATA") {
            format!(r#"{}\KeyMouse\edge_profile"#, appdata)
        } else if let Ok(temp) = std::env::var("TEMP") {
            format!(r#"{}\KeyMouse\edge_profile"#, temp)
        } else {
            r#"C:\ProgramData\KeyMouse\edge_profile"#.to_string()
        };
        let _ = std::fs::create_dir_all(&profile_dir);

        let edge_x86 = "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe";
        let edge_x64 = "C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe";
        let chrome_path = "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe";

        let mut desktop_name = to_wstring("WinSta0\\Default");

        let mut si: STARTUPINFOW = std::mem::zeroed();
        si.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
        si.lp_desktop = desktop_name.as_mut_ptr();

        let mut pi: PROCESS_INFORMATION = std::mem::zeroed();

        let cmd_str = if std::path::Path::new(edge_x86).exists() {
            format!(r#""{}" --user-data-dir="{}" --app="http://127.0.0.1:28888" --window-size=860,620 --no-first-run --no-default-browser-check --disable-features=Translate"#, edge_x86, profile_dir)
        } else if std::path::Path::new(edge_x64).exists() {
            format!(r#""{}" --user-data-dir="{}" --app="http://127.0.0.1:28888" --window-size=860,620 --no-first-run --no-default-browser-check --disable-features=Translate"#, edge_x64, profile_dir)
        } else if std::path::Path::new(chrome_path).exists() {
            format!(r#""{}" --user-data-dir="{}" --app="http://127.0.0.1:28888" --window-size=860,620 --no-first-run --no-default-browser-check --disable-features=Translate"#, chrome_path, profile_dir)
        } else {
            "cmd.exe /c start http://127.0.0.1:28888".to_string()
        };

        let mut cmd_w = to_wstring(&cmd_str);

        let res = CreateProcessW(
            std::ptr::null(),
            cmd_w.as_mut_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
            0,
            std::ptr::null_mut(),
            std::ptr::null(),
            &si,
            &mut pi,
        );

        if res != 0 {
            let pid = pi.dw_process_id;
            SPAWNED_PID.store(pid, Ordering::SeqCst);
            CloseHandle(pi.h_process);
            CloseHandle(pi.h_thread);

            thread::spawn(move || {
                attach_to_default_desktop();
                let mut found_count = 0;
                for _ in 0..40 {
                    thread::sleep(Duration::from_millis(150));
                    FOUND_HWND.store(0, Ordering::SeqCst);
                    let desk_name = to_wstring("Default");
                    let hdesk = OpenDesktopW(desk_name.as_ptr(), 0, 0, 0x01FF);
                    if hdesk != 0 {
                        EnumDesktopWindows(hdesk, enum_desktop_windows_proc, pid as isize);
                        CloseDesktop(hdesk);
                    }
                    let win = FOUND_HWND.load(Ordering::SeqCst);
                    if win != 0 {
                        UI_HWND.store(win, Ordering::SeqCst);
                        make_window_non_resizable(win);
                        found_count += 1;
                        if found_count >= 8 {
                            break;
                        }
                    }
                }
            });
        }
    }
}

unsafe fn make_window_non_resizable(hwnd: isize) {
    unsafe {
        let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
        let new_style = style & !(WS_THICKFRAME | WS_MAXIMIZEBOX);
        if style != new_style {
            SetWindowLongW(hwnd, GWL_STYLE, new_style as i32);
        }
        SetWindowPos(hwnd, 0, 0, 0, 860, 620, SWP_NOMOVE | SWP_NOZORDER | SWP_FRAMECHANGED);

        // Apply High-Res Taskbar Icon directly to the window
        let app_icon = APP_ICON_BIG.load(Ordering::Relaxed);
        let tray_icon = TRAY_ICON_ON.load(Ordering::Relaxed);
        if app_icon != 0 {
            SendMessageW(hwnd, WM_SETICON, ICON_BIG, app_icon);
        }
        if tray_icon != 0 {
            SendMessageW(hwnd, WM_SETICON, ICON_SMALL, tray_icon);
        }
    }
}

fn load_config() {
    if let Ok(content) = std::fs::read_to_string("keymouse_config.json") {
        apply_config_json(&content);
    }
}

fn save_config() {
    let content = get_config_json();
    let _ = std::fs::write("keymouse_config.json", content);
}

fn get_config_json() -> String {
    let rules = HOTKEY_RULES.lock().unwrap();
    let mut hotkeys_json = String::new();
    for (i, r) in rules.iter().enumerate() {
        if i > 0 { hotkeys_json.push_str(",\n    "); }
        hotkeys_json.push_str(&format!(
            r#"{{"combo": "{}", "enabled": {}, "action": "{}"}}"#,
            r.combo, r.enabled, r.action
        ));
    }

    let is_admin = unsafe { IsUserAnAdmin() != 0 };

    format!(
        r#"{{
  "mouse_mode": {},
  "is_admin": {},
  "lang": "{}",
  "min_speed": {:.1},
  "max_speed": {:.1},
  "accel_ms": {},
  "turbo_mult": {:.1},
  "enable_beep": {},
  "toast_pos": "{}",
  "hotkeys": [
    {}
  ],
  "bindings": {{
    "lclick": "{}",
    "rclick": "{}",
    "scroll_up": "{}",
    "scroll_down": "{}",
    "up": "{}",
    "down": "{}",
    "left": "{}",
    "right": "{}"
  }}
}}"#,
        MOUSE_MODE.load(Ordering::Relaxed),
        is_admin,
        get_lang(),
        MIN_SPEED_X10.load(Ordering::Relaxed) as f32 / 10.0,
        MAX_SPEED_X10.load(Ordering::Relaxed) as f32 / 10.0,
        ACCEL_TICKS.load(Ordering::Relaxed) * 8,
        TURBO_MULT_X10.load(Ordering::Relaxed) as f32 / 10.0,
        ENABLE_BEEP.load(Ordering::Relaxed),
        get_toast_pos(),
        hotkeys_json,
        vk_to_name(VK_BIND_LCLICK.load(Ordering::Relaxed)),
        vk_to_name(VK_BIND_RCLICK.load(Ordering::Relaxed)),
        vk_to_name(VK_BIND_SCROLL_UP.load(Ordering::Relaxed)),
        vk_to_name(VK_BIND_SCROLL_DOWN.load(Ordering::Relaxed)),
        vk_to_name(VK_BIND_UP.load(Ordering::Relaxed)),
        vk_to_name(VK_BIND_DOWN.load(Ordering::Relaxed)),
        vk_to_name(VK_BIND_LEFT.load(Ordering::Relaxed)),
        vk_to_name(VK_BIND_RIGHT.load(Ordering::Relaxed))
    )
}

fn extract_json_val<'a>(body: &'a str, key: &str) -> Option<&'a str> {
    let pattern = format!("\"{}\"", key);
    let k_pos = body.find(&pattern)?;
    let after_k = &body[k_pos + pattern.len()..];
    let colon_pos = after_k.find(':')?;
    let rest = after_k[colon_pos + 1..].trim_start();
    if rest.starts_with('"') {
        let end_quote = rest[1..].find('"')?;
        Some(&rest[1..1 + end_quote])
    } else {
        let end_val = rest.find(|c| c == ',' || c == '}' || c == '\r' || c == '\n').unwrap_or(rest.len());
        Some(rest[..end_val].trim())
    }
}

fn apply_config_json(body: &str) {
    if let Some(v) = extract_json_val(body, "lang") {
        set_lang(v);
        let curr_on = MOUSE_MODE.load(Ordering::Relaxed);
        update_tray_icon(curr_on);
    }
    if let Some(v) = extract_json_val(body, "min_speed") {
        if let Ok(f) = v.parse::<f32>() { MIN_SPEED_X10.store((f * 10.0) as u32, Ordering::Relaxed); }
    }
    if let Some(v) = extract_json_val(body, "max_speed") {
        if let Ok(f) = v.parse::<f32>() { MAX_SPEED_X10.store((f * 10.0) as u32, Ordering::Relaxed); }
    }
    if let Some(v) = extract_json_val(body, "accel_ms") {
        if let Ok(ms) = v.parse::<u32>() { ACCEL_TICKS.store((ms / 8).max(10), Ordering::Relaxed); }
    }
    if let Some(v) = extract_json_val(body, "turbo_mult") {
        if let Ok(f) = v.parse::<f32>() { TURBO_MULT_X10.store((f * 10.0) as u32, Ordering::Relaxed); }
    }
    if let Some(v) = extract_json_val(body, "enable_beep") {
        if let Ok(b) = v.parse::<bool>() { ENABLE_BEEP.store(b, Ordering::Relaxed); }
    }
    if let Some(v) = extract_json_val(body, "toast_pos") {
        set_toast_pos(v);
    }
    if let Some(v) = extract_json_val(body, "lclick") {
        VK_BIND_LCLICK.store(name_to_vk(v), Ordering::Relaxed);
    }
    if let Some(v) = extract_json_val(body, "rclick") {
        VK_BIND_RCLICK.store(name_to_vk(v), Ordering::Relaxed);
    }
    if let Some(v) = extract_json_val(body, "scroll_up") {
        VK_BIND_SCROLL_UP.store(name_to_vk(v), Ordering::Relaxed);
    }
    if let Some(v) = extract_json_val(body, "scroll_down") {
        VK_BIND_SCROLL_DOWN.store(name_to_vk(v), Ordering::Relaxed);
    }
    if let Some(v) = extract_json_val(body, "up") {
        VK_BIND_UP.store(name_to_vk(v), Ordering::Relaxed);
    }
    if let Some(v) = extract_json_val(body, "down") {
        VK_BIND_DOWN.store(name_to_vk(v), Ordering::Relaxed);
    }
    if let Some(v) = extract_json_val(body, "left") {
        VK_BIND_LEFT.store(name_to_vk(v), Ordering::Relaxed);
    }
    if let Some(v) = extract_json_val(body, "right") {
        VK_BIND_RIGHT.store(name_to_vk(v), Ordering::Relaxed);
    }

    if let Some(rules) = parse_hotkeys_json(body) {
        if !rules.is_empty() {
            let mut lock = HOTKEY_RULES.lock().unwrap();
            *lock = rules;
        }
    }
}

// Local Web Server for Fluent UI
fn start_web_server() {
    thread::spawn(|| {
        attach_to_default_desktop();
        let listener = match TcpListener::bind("127.0.0.1:28888") {
            Ok(l) => l,
            Err(_) => return,
        };

        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                let mut buffer = Vec::new();
                let mut temp = [0u8; 4096];
                let mut content_length = 0usize;
                let mut header_end = None;

                loop {
                    let n = match stream.read(&mut temp) {
                        Ok(n) if n > 0 => n,
                        _ => break,
                    };
                    buffer.extend_from_slice(&temp[..n]);

                    if header_end.is_none() {
                        if let Some(pos) = buffer.windows(4).position(|w| w == b"\r\n\r\n") {
                            header_end = Some(pos);
                            let header_str = String::from_utf8_lossy(&buffer[..pos]);
                            for line in header_str.lines() {
                                if line.to_ascii_lowercase().starts_with("content-length:") {
                                    if let Some(val_str) = line.split(':').nth(1) {
                                        content_length = val_str.trim().parse::<usize>().unwrap_or(0);
                                    }
                                }
                            }
                        }
                    }

                    if let Some(pos) = header_end {
                        if buffer.len() >= pos + 4 + content_length {
                            break;
                        }
                    }
                }

                if header_end.is_none() {
                    continue;
                }

                let pos = header_end.unwrap();
                let header_str = String::from_utf8_lossy(&buffer[..pos]);
                let body_str = String::from_utf8_lossy(&buffer[pos + 4..pos + 4 + content_length]);

                let first_line = header_str.lines().next().unwrap_or("");
                let parts: Vec<&str> = first_line.split_whitespace().collect();
                if parts.len() < 2 { continue; }
                let method = parts[0];
                let path = parts[1];

                if method == "GET" && (path == "/" || path == "/index.html") {
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        UI_HTML.len(),
                        UI_HTML
                    );
                    let _ = stream.write_all(resp.as_bytes());
                } else if method == "GET" && path == "/api/config" {
                    let json = get_config_json();
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        json.len(),
                        json
                    );
                    let _ = stream.write_all(resp.as_bytes());
                } else if method == "POST" && path == "/api/config" {
                    apply_config_json(&body_str);
                    save_config();
                    let resp = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 12\r\nConnection: close\r\n\r\n{\"ok\": true}";
                    let _ = stream.write_all(resp.as_bytes());
                } else if method == "POST" && path == "/api/toggle" {
                    let curr = MOUSE_MODE.load(Ordering::SeqCst);
                    set_mouse_mode(!curr);
                    let resp = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 12\r\nConnection: close\r\n\r\n{\"ok\": true}";
                    let _ = stream.write_all(resp.as_bytes());
                } else if method == "POST" && path == "/api/elevate" {
                    let mut success = false;
                    if let Ok(exe_path) = std::env::current_exe() {
                        let exe_w = to_wstring(&exe_path.to_string_lossy());
                        let op_w = to_wstring("runas");
                        let res = unsafe {
                            ShellExecuteW(
                                0,
                                op_w.as_ptr(),
                                exe_w.as_ptr(),
                                std::ptr::null(),
                                std::ptr::null(),
                                1,
                            )
                        };
                        if res > 32 {
                            success = true;
                        }
                    }
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{{\"ok\": {}}}",
                        if success { 12 } else { 13 },
                        success
                    );
                    let _ = stream.write_all(resp.as_bytes());
                    if success {
                        std::thread::spawn(|| {
                            std::thread::sleep(std::time::Duration::from_millis(500));
                            std::process::exit(0);
                        });
                    }
                } else if method == "GET" && path == "/api/status" {
                    let is_on = MOUSE_MODE.load(Ordering::Relaxed);
                    let json = format!("{{\"mouse_mode\": {}}}", is_on);
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        json.len(),
                        json
                    );
                    let _ = stream.write_all(resp.as_bytes());
                } else if method == "POST" && path == "/api/hide" {
                    let resp = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 12\r\nConnection: close\r\n\r\n{\"ok\": true}";
                    let _ = stream.write_all(resp.as_bytes());
                } else if method == "GET" && (path == "/icon-256.png" || path == "/icon.png") {
                    let resp_head = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        ICON_256_PNG.len()
                    );
                    let _ = stream.write_all(resp_head.as_bytes());
                    let _ = stream.write_all(ICON_256_PNG);
                } else if method == "GET" && path == "/icon-64.png" {
                    let resp_head = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        ICON_64_PNG.len()
                    );
                    let _ = stream.write_all(resp_head.as_bytes());
                    let _ = stream.write_all(ICON_64_PNG);
                } else if method == "GET" && path == "/favicon.ico" {
                    let resp_head = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: image/x-icon\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        FAVICON_ICO.len()
                    );
                    let _ = stream.write_all(resp_head.as_bytes());
                    let _ = stream.write_all(FAVICON_ICO);
                } else {
                    let resp = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
                    let _ = stream.write_all(resp.as_bytes());
                }
                let _ = stream.flush();
            }
        }
    });
}

// Low-Level Keyboard Hook
unsafe extern "system" fn keyboard_hook_proc(n_code: i32, w_param: usize, l_param: isize) -> isize {
    unsafe {
        if n_code >= 0 {
            let hook_struct = &*(l_param as *const KBDLLHOOKSTRUCT);
            let vk = hook_struct.vk_code;
            let is_down = w_param == WM_KEYDOWN || w_param == WM_SYSKEYDOWN;
            let is_up = w_param == WM_KEYUP || w_param == WM_SYSKEYUP;

            // Track physical modifier keys
            if vk == VK_SHIFT || vk == VK_LSHIFT || vk == VK_RSHIFT {
                SHIFT_HELD.store(is_down, Ordering::Relaxed);
            }
            if vk == VK_CONTROL || vk == VK_LCONTROL || vk == VK_RCONTROL {
                CTRL_HELD.store(is_down, Ordering::Relaxed);
            }
            if vk == VK_MENU || vk == VK_LMENU || vk == VK_RMENU {
                ALT_HELD.store(is_down, Ordering::Relaxed);
            }

            let shift_now = SHIFT_HELD.load(Ordering::Relaxed);
            let ctrl_now = CTRL_HELD.load(Ordering::Relaxed);
            let alt_now = ALT_HELD.load(Ordering::Relaxed);

            // ==================== DYNAMIC HOTKEY CHECKS ====================
            if is_down {
                let mut triggered_action: Option<bool> = None; // Some(true) = toggle, Some(false) = exit
                {
                    if let Ok(rules) = HOTKEY_RULES.lock() {
                        for rule in rules.iter() {
                            if !rule.enabled { continue; }

                            // Exit hotkeys (like Esc) should only be intercepted when mouse mode is ON
                            if rule.is_exit && !MOUSE_MODE.load(Ordering::Relaxed) {
                                continue;
                            }

                            let mut matched = false;

                            // 1. Shift + Alt chord
                            if rule.is_chord_shift_alt {
                                if (vk == VK_LMENU || vk == VK_RMENU || vk == VK_MENU) && shift_now {
                                    matched = true;
                                } else if (vk == VK_LSHIFT || vk == VK_RSHIFT || vk == VK_SHIFT) && alt_now {
                                    matched = true;
                                }
                            }
                            // 2. Alt + Space chord
                            else if rule.is_chord_alt_space {
                                if vk == VK_SPACE && alt_now {
                                    matched = true;
                                }
                            }
                            // 3. Ctrl + Shift chord
                            else if rule.is_chord_ctrl_shift {
                                if (vk == VK_LCONTROL || vk == VK_RCONTROL || vk == VK_CONTROL) && shift_now {
                                    matched = true;
                                } else if (vk == VK_LSHIFT || vk == VK_RSHIFT || vk == VK_SHIFT) && ctrl_now {
                                    matched = true;
                                }
                            }
                            // 4. Ctrl + Alt chord
                            else if rule.is_chord_ctrl_alt {
                                if (vk == VK_LCONTROL || vk == VK_RCONTROL || vk == VK_CONTROL) && alt_now {
                                    matched = true;
                                } else if (vk == VK_LMENU || vk == VK_RMENU || vk == VK_MENU) && ctrl_now {
                                    matched = true;
                                }
                            }
                            // 5. Generic Key + Modifiers (e.g. F8, Ctrl+Shift+M, Esc, etc.)
                            else if rule.trigger_vk != 0 && vk == rule.trigger_vk {
                                if (!rule.req_ctrl || ctrl_now) && (!rule.req_shift || shift_now) && (!rule.req_alt || alt_now) {
                                    matched = true;
                                }
                            }

                            if matched {
                                if rule.is_exit {
                                    triggered_action = Some(false);
                                } else {
                                    triggered_action = Some(true);
                                }
                                break;
                            }
                        }
                    }
                }

                if let Some(is_toggle) = triggered_action {
                    if is_toggle {
                        let curr = MOUSE_MODE.load(Ordering::SeqCst);
                        set_mouse_mode(!curr);
                        return 1;
                    } else {
                        if MOUSE_MODE.load(Ordering::SeqCst) {
                            set_mouse_mode(false);
                            return 1;
                        }
                    }
                }
            }

            // ==================== MOUSE CONTROL ACTIONS ====================
            if MOUSE_MODE.load(Ordering::SeqCst) {
                let bind_up = VK_BIND_UP.load(Ordering::Relaxed);
                let bind_down = VK_BIND_DOWN.load(Ordering::Relaxed);
                let bind_left = VK_BIND_LEFT.load(Ordering::Relaxed);
                let bind_right = VK_BIND_RIGHT.load(Ordering::Relaxed);
                let bind_lc = VK_BIND_LCLICK.load(Ordering::Relaxed);
                let bind_rc = VK_BIND_RCLICK.load(Ordering::Relaxed);
                let bind_su = VK_BIND_SCROLL_UP.load(Ordering::Relaxed);
                let bind_sd = VK_BIND_SCROLL_DOWN.load(Ordering::Relaxed);

                // Movement (only configured bindings, NO arrow keys hijacking)
                if vk == bind_up {
                    MOVE_UP.store(is_down, Ordering::Relaxed);
                    return 1;
                }
                if vk == bind_down {
                    MOVE_DOWN.store(is_down, Ordering::Relaxed);
                    return 1;
                }
                if vk == bind_left {
                    MOVE_LEFT.store(is_down, Ordering::Relaxed);
                    return 1;
                }
                if vk == bind_right {
                    MOVE_RIGHT.store(is_down, Ordering::Relaxed);
                    return 1;
                }

                // Left Click & Drag
                if vk == bind_lc {
                    if is_down {
                        if !LEFT_HELD.swap(true, Ordering::Relaxed) {
                            mouse_event(MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0);
                        }
                    } else if is_up {
                        if LEFT_HELD.swap(false, Ordering::Relaxed) {
                            mouse_event(MOUSEEVENTF_LEFTUP, 0, 0, 0, 0);
                        }
                    }
                    return 1;
                }

                // Right Click
                if vk == bind_rc {
                    if is_down {
                        if !RIGHT_HELD.swap(true, Ordering::Relaxed) {
                            mouse_event(MOUSEEVENTF_RIGHTDOWN, 0, 0, 0, 0);
                        }
                    } else if is_up {
                        if RIGHT_HELD.swap(false, Ordering::Relaxed) {
                            mouse_event(MOUSEEVENTF_RIGHTUP, 0, 0, 0, 0);
                        }
                    }
                    return 1;
                }

                // Scroll Up / Down
                if vk == bind_su {
                    SCROLL_UP.store(is_down, Ordering::Relaxed);
                    return 1;
                }
                if vk == bind_sd {
                    SCROLL_DOWN.store(is_down, Ordering::Relaxed);
                    return 1;
                }
            }
        }

        CallNextHookEx(0, n_code, w_param, l_param)
    }
}

fn start_cursor_physics_thread() {
    thread::spawn(|| {
        attach_to_default_desktop();
        let mut ticks_x: u32 = 0;
        let mut ticks_y: u32 = 0;
        let mut last_dir_x: i32 = 0;
        let mut last_dir_y: i32 = 0;

        let mut vel_x: f32 = 0.0;
        let mut vel_y: f32 = 0.0;

        let mut accum_x: f32 = 0.0;
        let mut accum_y: f32 = 0.0;
        let mut scroll_ticks: u32 = 0;

        loop {
            thread::sleep(Duration::from_millis(8)); // 125 FPS

            if !MOUSE_MODE.load(Ordering::Relaxed) {
                ticks_x = 0;
                ticks_y = 0;
                last_dir_x = 0;
                last_dir_y = 0;
                vel_x = 0.0;
                vel_y = 0.0;
                accum_x = 0.0;
                accum_y = 0.0;
                scroll_ticks = 0;
                continue;
            }

            let up = MOVE_UP.load(Ordering::Relaxed);
            let down = MOVE_DOWN.load(Ordering::Relaxed);
            let left = MOVE_LEFT.load(Ordering::Relaxed);
            let right = MOVE_RIGHT.load(Ordering::Relaxed);

            let dir_x = (right as i32) - (left as i32);
            let dir_y = (down as i32) - (up as i32);

            let min_speed = MIN_SPEED_X10.load(Ordering::Relaxed) as f32 / 10.0;
            let max_speed = MAX_SPEED_X10.load(Ordering::Relaxed) as f32 / 10.0;
            let accel_ticks = ACCEL_TICKS.load(Ordering::Relaxed).max(10) as f32;
            let turbo_mult = TURBO_MULT_X10.load(Ordering::Relaxed) as f32 / 10.0;

            // Reset acceleration ticks if direction changed on that axis
            if dir_x != last_dir_x {
                ticks_x = 0;
                last_dir_x = dir_x;
            }
            if dir_y != last_dir_y {
                ticks_y = 0;
                last_dir_y = dir_y;
            }

            let mut target_vx = 0.0f32;
            if dir_x != 0 {
                ticks_x = ticks_x.saturating_add(1);
                let tx = (ticks_x as f32 / accel_ticks).min(1.0);
                let speed_x = min_speed + (max_speed - min_speed) * (tx * tx);
                target_vx = (dir_x as f32) * speed_x;
            }

            let mut target_vy = 0.0f32;
            if dir_y != 0 {
                ticks_y = ticks_y.saturating_add(1);
                let ty = (ticks_y as f32 / accel_ticks).min(1.0);
                let speed_y = min_speed + (max_speed - min_speed) * (ty * ty);
                target_vy = (dir_y as f32) * speed_y;
            }

            // Diagonal normalization
            if dir_x != 0 && dir_y != 0 {
                let norm = 0.70710678f32;
                target_vx *= norm;
                target_vy *= norm;
            }

            // Modifiers: Shift = Turbo, Ctrl = Precision Slow
            if SHIFT_HELD.load(Ordering::Relaxed) {
                target_vx *= turbo_mult;
                target_vy *= turbo_mult;
            }
            if CTRL_HELD.load(Ordering::Relaxed) {
                if dir_x != 0 {
                    target_vx = (dir_x as f32) * (min_speed * 0.35).max(0.25);
                }
                if dir_y != 0 {
                    target_vy = (dir_y as f32) * (min_speed * 0.35).max(0.25);
                }
            }

            // Critically damped exponential smoothing:
            // On movement: 0.45 factor reaches 95% target in ~35ms (snappy, low-latency)
            // On release: 0.58 factor brings to crisp stop in ~24ms (no overshoot)
            let smooth_x = if dir_x != 0 { 0.45f32 } else { 0.58f32 };
            let smooth_y = if dir_y != 0 { 0.45f32 } else { 0.58f32 };

            vel_x += (target_vx - vel_x) * smooth_x;
            vel_y += (target_vy - vel_y) * smooth_y;

            if dir_x == 0 && vel_x.abs() < 0.05 {
                vel_x = 0.0;
            }
            if dir_y == 0 && vel_y.abs() < 0.05 {
                vel_y = 0.0;
            }

            if dir_x == 0 && dir_y == 0 && vel_x == 0.0 && vel_y == 0.0 {
                accum_x = 0.0;
                accum_y = 0.0;
            } else {
                accum_x += vel_x;
                accum_y += vel_y;

                let step_x = accum_x.trunc() as i32;
                let step_y = accum_y.trunc() as i32;

                accum_x -= step_x as f32;
                accum_y -= step_y as f32;

                if step_x != 0 || step_y != 0 {
                    unsafe {
                        mouse_event(MOUSEEVENTF_MOVE, step_x as u32, step_y as u32, 0, 0);
                    }
                }
            }

            let s_up = SCROLL_UP.load(Ordering::Relaxed);
            let s_down = SCROLL_DOWN.load(Ordering::Relaxed);

            if s_up || s_down {
                scroll_ticks = scroll_ticks.saturating_add(1);
                if scroll_ticks == 1 || scroll_ticks % 8 == 0 {
                    let mult = if SHIFT_HELD.load(Ordering::Relaxed) { 3 } else { 1 };
                    let delta = if s_up {
                        WHEEL_DELTA * mult
                    } else {
                        -WHEEL_DELTA * mult
                    };
                    unsafe {
                        mouse_event(MOUSEEVENTF_WHEEL, 0, 0, delta as u32, 0);
                    }
                }
            } else {
                scroll_ticks = 0;
            }
        }
    });
}

unsafe fn render_and_update_toast(hwnd: isize, is_on: bool) {
    unsafe {
        let w = 290i32;
        let h = 44i32;

        let mem_dc = CreateCompatibleDC(0);
        if mem_dc == 0 { return; }

        let mut bmi: BITMAPINFO = std::mem::zeroed();
        bmi.bmi_header.bi_size = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bmi.bmi_header.bi_width = w;
        bmi.bmi_header.bi_height = -h; // top-down
        bmi.bmi_header.bi_planes = 1;
        bmi.bmi_header.bi_bit_count = 32;
        bmi.bmi_header.bi_compression = 0;

        let mut bits: *mut u8 = std::ptr::null_mut();
        let dib = CreateDIBSection(mem_dc, &bmi, 0, &mut bits, 0, 0);
        if dib == 0 || bits.is_null() {
            DeleteDC(mem_dc);
            return;
        }

        let old_bmp = SelectObject(mem_dc, dib);

        // 1. Signed Distance Field for perfectly anti-aliased pill border
        let hw = (w as f32) / 2.0 - 1.5;
        let hh = (h as f32) / 2.0 - 1.5;
        let r = 20.0f32;

        let (border_r, border_g, border_b) = if is_on {
            (0u8, 210u8, 255u8) // Cyan
        } else {
            (245u8, 158u8, 11u8) // Amber
        };
        let (dot_r, dot_g, dot_b) = if is_on {
            (16u8, 185u8, 129u8) // Emerald Green
        } else {
            (245u8, 158u8, 11u8) // Amber
        };

        let bg_r = 15.0f32;
        let bg_g = 19.0f32;
        let bg_b = 30.0f32;

        let slice = std::slice::from_raw_parts_mut(bits, (w * h * 4) as usize);

        for y in 0..h {
            for x in 0..w {
                let idx = ((y * w + x) * 4) as usize;
                let px = (x as f32 + 0.5) - (w as f32 / 2.0);
                let py = (y as f32 + 0.5) - (h as f32 / 2.0);
                let qx = px.abs() - (hw - r);
                let qy = py.abs() - (hh - r);
                let dist = qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0) - r;

                if dist > 1.0 {
                    slice[idx] = 0;
                    slice[idx + 1] = 0;
                    slice[idx + 2] = 0;
                    slice[idx + 3] = 0;
                    continue;
                }

                if dist >= -1.8 {
                    let edge = (1.0 - dist).max(0.0).min(1.0);
                    let a = (edge * 255.0) as u8;
                    slice[idx] = ((border_b as f32) * edge) as u8;
                    slice[idx + 1] = ((border_g as f32) * edge) as u8;
                    slice[idx + 2] = ((border_r as f32) * edge) as u8;
                    slice[idx + 3] = a;
                } else if dist >= -3.0 {
                    let t = (dist - (-3.0)) / 1.2;
                    let cr = bg_r * (1.0 - t) + (border_r as f32) * t;
                    let cg = bg_g * (1.0 - t) + (border_g as f32) * t;
                    let cb = bg_b * (1.0 - t) + (border_b as f32) * t;
                    let a = 245u8;
                    slice[idx] = (cb * 245.0 / 255.0) as u8;
                    slice[idx + 1] = (cg * 245.0 / 255.0) as u8;
                    slice[idx + 2] = (cr * 245.0 / 255.0) as u8;
                    slice[idx + 3] = a;
                } else {
                    let a = 242u8;
                    slice[idx] = (bg_b * 242.0 / 255.0) as u8;
                    slice[idx + 1] = (bg_g * 242.0 / 255.0) as u8;
                    slice[idx + 2] = (bg_r * 242.0 / 255.0) as u8;
                    slice[idx + 3] = a;
                }
            }
        }

        // 2. Anti-aliased Status Dot
        let dot_cx = 19.0f32;
        let dot_cy = 22.0f32;
        for y in 10..34 {
            for x in 9..31 {
                let idx = ((y * w + x) * 4) as usize;
                let d = (x as f32 - dot_cx).hypot(y as f32 - dot_cy) - 4.5;
                if d <= 0.5 {
                    let cov = (0.5 - d).max(0.0).min(1.0);
                    let old_b = slice[idx] as f32;
                    let old_g = slice[idx + 1] as f32;
                    let old_r = slice[idx + 2] as f32;
                    slice[idx] = (old_b * (1.0 - cov) + (dot_b as f32) * cov) as u8;
                    slice[idx + 1] = (old_g * (1.0 - cov) + (dot_g as f32) * cov) as u8;
                    slice[idx + 2] = (old_r * (1.0 - cov) + (dot_r as f32) * cov) as u8;
                    slice[idx + 3] = 255;
                }
            }
        }

        // 3. ClearType Typography via GDI
        SetBkMode(mem_dc, 1);
        let font_face = to_wstring("Segoe UI");
        let title_font = CreateFontW(15, 0, 0, 0, 700, 0, 0, 0, 1, 0, 0, 5, 0, font_face.as_ptr());
        let sub_font = CreateFontW(12, 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 5, 0, font_face.as_ptr());

        let old_font = SelectObject(mem_dc, title_font);
        SetTextColor(mem_dc, 0x00FFFFFF);
        let is_en = get_lang() == "en";
        let title_str = if is_en {
            if is_on {
                to_wstring("KeyMouse: ENABLED (ON)")
            } else {
                to_wstring("KeyMouse: DISABLED (OFF)")
            }
        } else {
            if is_on {
                to_wstring("KeyMouse: ĐÃ BẬT (ON)")
            } else {
                to_wstring("KeyMouse: ĐÃ TẮT (OFF)")
            }
        };
        let mut r_title = RECT { left: 32, top: 6, right: 285, bottom: 23 };
        DrawTextW(mem_dc, title_str.as_ptr(), -1, &mut r_title, DT_SINGLELINE | DT_NOPREFIX | DT_VCENTER);

        SelectObject(mem_dc, sub_font);
        let sub_color = if is_on { 0x00FFD200 } else { 0x00AFA39C };
        SetTextColor(mem_dc, sub_color);
        let sub_str = if is_en {
            if is_on {
                to_wstring("WASD move • Space left click • X right")
            } else {
                to_wstring("Keyboard normal typing • Press hotkey to enable")
            }
        } else {
            if is_on {
                to_wstring("WASD di chuyển • Space chuột trái • X phải")
            } else {
                to_wstring("Bàn phím gõ bình thường • Bấm phím tắt để bật")
            }
        };
        let mut r_sub = RECT { left: 32, top: 23, right: 285, bottom: 39 };
        DrawTextW(mem_dc, sub_str.as_ptr(), -1, &mut r_sub, DT_SINGLELINE | DT_NOPREFIX | DT_VCENTER);

        SelectObject(mem_dc, old_font);
        DeleteObject(title_font);
        DeleteObject(sub_font);

        // 4. Alpha channel fixup for text pixels
        for y in 5..40 {
            for x in 30..285 {
                let idx = ((y * w + x) * 4) as usize;
                let b = slice[idx];
                let g = slice[idx + 1];
                let r = slice[idx + 2];
                if r > 30 || g > 35 || b > 45 {
                    slice[idx + 3] = 255;
                }
            }
        }

        // 5. Update Layered Window with subpixel anti-aliasing
        let pos_str = get_toast_pos();
        let (tx, ty) = calc_toast_pos(&pos_str);
        let pt_dst = POINT { x: tx, y: ty };
        let size = POINT { x: w, y: h };
        let pt_src = POINT { x: 0, y: 0 };
        let blend = BLENDFUNCTION {
            blend_op: AC_SRC_OVER,
            blend_flags: 0,
            source_constant_alpha: 255,
            alpha_format: AC_SRC_ALPHA,
        };

        let screen_dc = GetDC(0);
        UpdateLayeredWindow(
            hwnd,
            screen_dc,
            &pt_dst,
            &size,
            mem_dc,
            &pt_src,
            0,
            &blend,
            ULW_ALPHA,
        );
        if screen_dc != 0 {
            ReleaseDC(0, screen_dc);
        }

        SelectObject(mem_dc, old_bmp);
        DeleteObject(dib);
        DeleteDC(mem_dc);
    }
}

// Create 32-bit Alpha HICON from raw BGRA pixel slice
unsafe fn create_hicon_from_bgra(w: i32, h: i32, bgra_data: &[u8]) -> isize {
    unsafe {
        let hdc = CreateCompatibleDC(0);
        if hdc == 0 { return 0; }

        let mut bmi: BITMAPINFO = std::mem::zeroed();
        bmi.bmi_header.bi_size = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bmi.bmi_header.bi_width = w;
        bmi.bmi_header.bi_height = -h; // top-down
        bmi.bmi_header.bi_planes = 1;
        bmi.bmi_header.bi_bit_count = 32;
        bmi.bmi_header.bi_compression = 0;

        let mut bits: *mut u8 = std::ptr::null_mut();
        let hbm_color = CreateDIBSection(hdc, &bmi, 0, &mut bits, 0, 0);
        if hbm_color == 0 || bits.is_null() {
            DeleteDC(hdc);
            return 0;
        }

        let slice = std::slice::from_raw_parts_mut(bits, (w * h * 4) as usize);
        let copy_len = slice.len().min(bgra_data.len());
        slice[..copy_len].copy_from_slice(&bgra_data[..copy_len]);

        let mask_bytes = vec![0u8; ((w + 31) / 32 * 4 * h) as usize];
        let hbm_mask = CreateBitmap(w, h, 1, 1, mask_bytes.as_ptr());

        let ii = ICONINFO {
            f_icon: 1,
            x_hotspot: 0,
            y_hotspot: 0,
            hbm_mask,
            hbm_color,
        };

        let hicon = CreateIconIndirect(&ii);

        DeleteObject(hbm_color);
        DeleteObject(hbm_mask);
        DeleteDC(hdc);

        hicon
    }
}

// Generate High-Resolution Mouse HICON for Windows Tray (Hình Cũ, +20% enlarged, exact DPI metric)
unsafe fn create_mouse_hicon(is_on: bool, target_size: i32) -> isize {
    unsafe {
        if target_size <= 16 {
            let raw = if is_on {
                TRAY_ON_16_RAW.as_slice()
            } else {
                TRAY_OFF_16_RAW.as_slice()
            };
            create_hicon_from_bgra(16, 16, raw)
        } else {
            let raw = if is_on {
                TRAY_ON_32_RAW.as_slice()
            } else {
                TRAY_OFF_32_RAW.as_slice()
            };
            create_hicon_from_bgra(32, 32, raw)
        }
    }
}

// Win32 Floating OSD Toast Window Proc
unsafe extern "system" fn toast_wnd_proc(hwnd: isize, msg: u32, w_param: usize, l_param: isize) -> isize {
    unsafe {
        match msg {
            WM_SHOW_TOAST => {
                let is_on = w_param != 0;
                TOAST_IS_ON.store(is_on, Ordering::SeqCst);

                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                TOAST_HIDE_TIME.store(now + 1800, Ordering::SeqCst);

                render_and_update_toast(hwnd, is_on);
                SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW);
                SetTimer(hwnd, 1, 80, 0);
                0
            }
            WM_TIMER => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                let hide_at = TOAST_HIDE_TIME.load(Ordering::SeqCst);
                if now >= hide_at {
                    KillTimer(hwnd, 1);
                    ShowWindow(hwnd, SW_HIDE);
                }
                0
            }
            _ => DefWindowProcW(hwnd, msg, w_param, l_param),
        }
    }
}

// Win32 Hidden Tray Window Proc
unsafe extern "system" fn tray_wnd_proc(hwnd: isize, msg: u32, w_param: usize, l_param: isize) -> isize {
    unsafe {
        match msg {
            WM_COMMAND => {
                let id = (w_param & 0xFFFF) as usize;
                match id {
                    IDM_TOGGLE => {
                        let curr = MOUSE_MODE.load(Ordering::SeqCst);
                        set_mouse_mode(!curr);
                    }
                    IDM_OPEN_UI => {
                        open_fluent_ui();
                    }
                    IDM_EXIT => {
                        DestroyWindow(hwnd);
                    }
                    _ => {}
                }
                0
            }
            WM_TRAYICON => {
                let event = l_param as usize;
                if event == WM_RBUTTONUP || event == WM_CONTEXTMENU {
                    let hmenu = CreatePopupMenu();
                    let toggle_text = if MOUSE_MODE.load(Ordering::Relaxed) {
                        "⚡ Tắt chế độ chuột"
                    } else {
                        "⚡ Bật chế độ chuột"
                    };
                    AppendMenuW(hmenu, MF_STRING, IDM_TOGGLE, to_wstring(toggle_text).as_ptr());
                    AppendMenuW(hmenu, MF_STRING, IDM_OPEN_UI, to_wstring("🎨 Mở Giao Diện Fluent UI").as_ptr());
                    AppendMenuW(hmenu, MF_SEPARATOR, 0, std::ptr::null());
                    AppendMenuW(hmenu, MF_STRING, IDM_EXIT, to_wstring("❌ Thoát (Exit)").as_ptr());

                    let mut pt = POINT { x: 0, y: 0 };
                    GetCursorPos(&mut pt);
                    SetForegroundWindow(hwnd);
                    TrackPopupMenu(hmenu, TPM_RIGHTBUTTON, pt.x, pt.y, 0, hwnd, std::ptr::null());
                    DestroyMenu(hmenu);
                } else if event == WM_LBUTTONUP || event == WM_LBUTTONDBLCLK || event == NIN_SELECT || event == NIN_KEYSELECT {
                    open_fluent_ui();
                }
                0
            }
            WM_UPDATE_TRAY => {
                let is_on = w_param != 0;
                let hicon = if is_on {
                    TRAY_ICON_ON.load(Ordering::SeqCst)
                } else {
                    TRAY_ICON_OFF.load(Ordering::SeqCst)
                };
                if hicon != 0 {
                    let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
                    nid.cb_size = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
                    nid.h_wnd = hwnd;
                    nid.u_id = ID_TRAY_ICON;
                    nid.u_flags = NIF_ICON | NIF_TIP;
                    nid.h_icon = hicon;
                    let is_en = get_lang() == "en";
                    let tip_str = if is_en {
                        if is_on {
                            "KeyMouse Fluent: [ON] - Double-click to open settings"
                        } else {
                            "KeyMouse Fluent: [OFF] - Double-click to open settings"
                        }
                    } else {
                        if is_on {
                            "KeyMouse Fluent: [ĐÃ BẬT] - Đúp chuột để mở giao diện"
                        } else {
                            "KeyMouse Fluent: [ĐÃ TẮT] - Đúp chuột để mở giao diện"
                        }
                    };
                    let tip = to_wstring(tip_str);
                    for (i, &ch) in tip.iter().take(127).enumerate() {
                        nid.sz_tip[i] = ch;
                    }
                    Shell_NotifyIconW(NIM_MODIFY, &nid);
                }
                0
            }
            WM_DESTROY => {
                let ui = UI_HWND.load(Ordering::SeqCst);
                if ui != 0 && IsWindow(ui) != 0 {
                    PostMessageW(ui, WM_CLOSE, 0, 0);
                }
                let toast = TOAST_HWND.load(Ordering::SeqCst);
                if toast != 0 {
                    DestroyWindow(toast);
                }
                let hook = HOOK_HANDLE.load(Ordering::SeqCst);
                if hook != 0 {
                    UnhookWindowsHookEx(hook);
                }
                let icon_on = TRAY_ICON_ON.load(Ordering::SeqCst);
                if icon_on != 0 {
                    DestroyIcon(icon_on);
                }
                let icon_off = TRAY_ICON_OFF.load(Ordering::SeqCst);
                if icon_off != 0 {
                    DestroyIcon(icon_off);
                }
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(hwnd, msg, w_param, l_param),
        }
    }
}

fn check_and_request_elevation() {
    let args: Vec<String> = std::env::args().collect();
    let has_no_admin = args.iter().any(|a| a == "--no-admin");
    let is_elevated_flag = args.iter().any(|a| a == "--elevated");

    if !has_no_admin && !is_elevated_flag && unsafe { IsUserAnAdmin() } == 0 {
        if let Ok(exe_path) = std::env::current_exe() {
            let exe_w = to_wstring(&exe_path.to_string_lossy());
            let op_w = to_wstring("runas");
            let params_w = to_wstring("--elevated");
            let res = unsafe {
                ShellExecuteW(
                    0,
                    op_w.as_ptr(),
                    exe_w.as_ptr(),
                    params_w.as_ptr(),
                    std::ptr::null(),
                    1,
                )
            };
            if res > 32 {
                std::process::exit(0);
            }
        }
    }
}

fn main() {
    check_and_request_elevation();
    init_default_hotkeys();
    load_config();
    attach_to_default_desktop();

    // Start Local Web Server
    start_web_server();

    // Start 125 FPS Mouse Physics
    start_cursor_physics_thread();

    unsafe {
        let h_instance = GetModuleHandleW(std::ptr::null());
        let class_name = to_wstring("KeyMouseTrayClass");

        // Initialize Dynamic High-Resolution Mouse Tray & App Icons
        let sm_size = GetSystemMetrics(49); // SM_CXSMICON
        let icon_sz = if sm_size > 0 { sm_size } else { 16 };

        let hicon_on = create_mouse_hicon(true, icon_sz);
        let hicon_off = create_mouse_hicon(false, icon_sz);
        TRAY_ICON_ON.store(hicon_on, Ordering::SeqCst);
        TRAY_ICON_OFF.store(hicon_off, Ordering::SeqCst);

        let hicon_app_48 = create_hicon_from_bgra(48, 48, APP_ICON_48_RAW.as_slice());
        APP_ICON_BIG.store(hicon_app_48, Ordering::SeqCst);

        let initial_on = MOUSE_MODE.load(Ordering::Relaxed);
        let current_hicon = if initial_on { hicon_on } else { hicon_off };

        let wc = WNDCLASSEXW {
            cb_size: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_DBLCLKS,
            lpfn_wnd_proc: tray_wnd_proc,
            cb_cls_extra: 0,
            cb_wnd_extra: 0,
            h_instance,
            h_icon: current_hicon,
            h_cursor: LoadCursorW(0, 32512 as *const u16),
            h_br_background: 0,
            lpsz_menu_name: std::ptr::null(),
            lpsz_class_name: class_name.as_ptr(),
            h_icon_sm: current_hicon,
        };

        RegisterClassExW(&wc);

        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            to_wstring("KeyMouseHost").as_ptr(),
            0,
            0, 0, 0, 0,
            0, 0, h_instance,
            std::ptr::null_mut(),
        );

        MAIN_HWND.store(hwnd, Ordering::SeqCst);

        // Floating OSD Toast Window
        let toast_class = to_wstring("KeyMouseToastClass");
        let wc_toast = WNDCLASSEXW {
            cb_size: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfn_wnd_proc: toast_wnd_proc,
            cb_cls_extra: 0,
            cb_wnd_extra: 0,
            h_instance,
            h_icon: 0,
            h_cursor: LoadCursorW(0, 32512 as *const u16),
            h_br_background: 0,
            lpsz_menu_name: std::ptr::null(),
            lpsz_class_name: toast_class.as_ptr(),
            h_icon_sm: 0,
        };
        RegisterClassExW(&wc_toast);

        let screen_w = GetSystemMetrics(0);
        let toast_w = 290;
        let toast_h = 44;
        let toast_x = (screen_w - toast_w) / 2;

        let toast_hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_LAYERED | WS_EX_TRANSPARENT,
            toast_class.as_ptr(),
            to_wstring("KM_OSD_Toast").as_ptr(),
            WS_POPUP,
            toast_x, 42, toast_w, toast_h,
            0, 0, h_instance,
            std::ptr::null_mut(),
        );

        if toast_hwnd != 0 {
            TOAST_HWND.store(toast_hwnd, Ordering::SeqCst);
        }

        // System Tray Icon
        let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
        nid.cb_size = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.h_wnd = hwnd;
        nid.u_id = ID_TRAY_ICON;
        nid.u_flags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
        nid.u_callback_message = WM_TRAYICON;
        nid.h_icon = current_hicon;
        let is_en = get_lang() == "en";
        let tip_str = if is_en {
            if initial_on {
                "KeyMouse Fluent: [ON] - Double-click to open settings"
            } else {
                "KeyMouse Fluent: [OFF] - Double-click to open settings"
            }
        } else {
            if initial_on {
                "KeyMouse Fluent: [ĐÃ BẬT] - Đúp chuột để mở giao diện"
            } else {
                "KeyMouse Fluent: [ĐÃ TẮT] - Đúp chuột để mở giao diện"
            }
        };
        let tip = to_wstring(tip_str);
        for (i, &ch) in tip.iter().take(127).enumerate() {
            nid.sz_tip[i] = ch;
        }
        Shell_NotifyIconW(NIM_ADD, &nid);

        // Keyboard Hook
        let hook = SetWindowsHookExW(WH_KEYBOARD_LL, keyboard_hook_proc, 0, 0);
        if hook != 0 {
            HOOK_HANDLE.store(hook, Ordering::SeqCst);
        }

        // Open Fluent UI Window automatically on launch
        open_fluent_ui();

        // Message Pump
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, 0, 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        Shell_NotifyIconW(NIM_DELETE, &nid);
    }
}
