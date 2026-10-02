# KeyMouse Fluent Design 🖱️⌨️

<p align="center">
  <img src="src/icon-256.png" width="128" height="128" alt="KeyMouse Logo" />
</p>

<p align="center">
  <strong>Ultra-low latency, physics-based keyboard-to-mouse controller for Windows 11.</strong><br/>
  <em>Điều khiển chuột bằng bàn phím độ trễ cực thấp, vật lý gia tốc mượt mà dành cho Windows 11.</em>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Language-Rust_2024-orange.svg" alt="Rust" />
  <img src="https://img.shields.io/badge/Platform-Windows_11_%2F_10-0078D6.svg" alt="Windows" />
  <img src="https://img.shields.io/badge/Dependencies-Zero_External_Crates-brightgreen.svg" alt="Zero Dependencies" />
  <img src="https://img.shields.io/badge/Physics-125_FPS_Subpixel-blueviolet.svg" alt="125 FPS" />
  <img src="https://img.shields.io/badge/License-MIT-lightgrey.svg" alt="License" />
</p>

---

<p align="center">
  <a href="#english">English</a> • <a href="#tiếng-việt">Tiếng Việt</a>
</p>

---

<a name="english"></a>
## 🇬🇧 English

### Overview

**KeyMouse Fluent Design** is a lightweight, high-performance Windows utility built in **pure Rust** with zero third-party crate dependencies. It transforms your keyboard into a fluid, subpixel-accurate mouse replacement using low-level Win32 hooks (`WH_KEYBOARD_LL`) and GDI/User32 APIs.

Whether your physical mouse ran out of battery, you suffer from RSI/wrist fatigue, or you want high-precision pixel nudging without lifting your hands from the home row, KeyMouse delivers a first-class experience with a Windows 11 Fluent Design dashboard.

### Key Features

- **🚀 125 FPS Subpixel Physics Engine**:
  - Continuous velocity integration with subpixel fractional accumulators.
  - Natural acceleration curve: configurable Minimum Speed, Maximum Speed, Acceleration Ramp Time (ms), and Turbo multiplier.
  - Multi-key directional vector summation (supports simultaneous diagonals like `W+D`, `S+A`).
- **🎯 Full Drag & Drop / Region Selection**:
  - Spacebar acts as a true physical Left Click (`MOUSEEVENTF_LEFTDOWN` / `MOUSEEVENTF_LEFTUP`).
  - Hold Space to select text, drag windows, draw in graphics software, or select screenshot areas in Snipping Tool / Lightshot.
  - Dedicated Right Click (`X`) and separated Scroll Up (`Q`) / Scroll Down (`E`) controls.
- **✨ ClearType Anti-Aliased OSD Toast**:
  - Native layered floating Win32 window (`WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST`).
  - Smooth alpha-blended rounded pill shape with ClearType typography.
  - Customizable screen position: `Top-Center`, `Top-Right`, `Bottom-Center`, or `Bottom-Right`.
  - Shows real-time mode transitions and active hotkey statuses.
- **🎨 Windows 11 Fluent Design Dashboard**:
  - Embedded local micro-server (`127.0.0.1:28888`) launched as a native Microsoft Edge App window (`--app`).
  - Dark glassmorphism with acrylic blur effects.
  - Interactive ergonomic mouse diagram with dynamic SVG leader lines mapping keyboard keys to physical mouse buttons.
  - Visual D-Pad with live WASD movement feedback.
  - Interactive Hotkey Recorder for quick customization.
- **🌐 1-Click Bilingual Support**:
  - Instant toggle between English (`EN`) and Vietnamese (`VI`) across both the GUI dashboard and system OSD toasts.
- **🔔 High-DPI System Tray & Dynamic Icons**:
  - Custom high-resolution ergonomic mouse silhouette icons (16x16, 32x32, 48x48, 64x64, 256x256).
  - Tray icon visually reflects current status (Green LED when Active, Muted Gray when Inactive).
  - Quick access tray menu: Open Settings, Toggle On/Off, Exit.
- **🛡️ Smart Key Pass-Through**:
  - Arrow keys are never hijacked; standard typing and cursor navigation remain fully functional.
  - `Esc` key cleanly closes the settings dashboard when focused without intercepting system `Esc` actions.
- **⚡ Administrator Mode & Task Manager Support**:
  - Automatically requests or enables Administrator privileges to bypass Windows UIPI (User Interface Privilege Isolation).
  - Uninterrupted keyboard capture and mouse control across all elevated applications, including **Task Manager** (even with "Always on top" enabled), Registry Editor, and Admin Terminals.
  - Clickable "Run as Admin" badge right in the dashboard header for instant 1-click elevation.

### Default Shortcuts

| Action | Default Key | Note |
| :--- | :--- | :--- |
| **Toggle Mouse Mode** | `F8` | Global hotkey (customizable) |
| **Move Up / Down / Left / Right** | `W` / `S` / `A` / `D` | Supports smooth diagonals |
| **Left Click / Hold to Drag** | `Space` | Press & hold for drag/select |
| **Right Click** | `X` | Context menu |
| **Scroll Up** | `Q` | Wheel scroll up |
| **Scroll Down** | `E` | Wheel scroll down |
| **Close Dashboard** | `Esc` | Closes active settings window |

### Getting Started

#### Prerequisites
- Windows 10 or Windows 11 (64-bit).
- [Rust Toolchain](https://rustup.rs/) (edition 2024 or later).
- Microsoft Edge (pre-installed on Windows 10/11) for the Fluent UI app window.

#### Build from Source

```powershell
# 1. Clone the repository
git clone https://github.com/phatMT97/keymouse.git
cd keymouse

# 2. Build the optimized release binary
cargo build --release

# 3. Run KeyMouse
./target/release/keymouse.exe
```

The compiled executable is a standalone, lightweight binary (~1.5 MB stripped) with no external DLL dependencies.

### Configuration (`keymouse_config.json`)

KeyMouse automatically saves your settings to `keymouse_config.json`:

```json
{
  "mouse_mode": true,
  "lang": "en",
  "min_speed": 0.8,
  "max_speed": 8.0,
  "accel_ms": 450,
  "turbo_mult": 2.0,
  "enable_beep": true,
  "toast_pos": "top-center",
  "hotkeys": [
    {"combo": "F8", "enabled": true, "action": "toggle"},
    {"combo": "Esc", "enabled": true, "action": "exit"}
  ],
  "bindings": {
    "lclick": "Space",
    "rclick": "X",
    "scroll_up": "Q",
    "scroll_down": "E",
    "up": "W",
    "down": "S",
    "left": "A",
    "right": "D"
  }
}
```

---

<a name="tiếng-việt"></a>
## 🇻🇳 Tiếng Việt

### Giới thiệu

**KeyMouse Fluent Design** là phần mềm điều khiển chuột bằng bàn phím hiệu năng cao dành cho Windows 11, được viết bằng **Rust thuần** (Pure Rust) và gọi trực tiếp các API Win32 (GDI, User32, Low-level Hooks) mà **không phụ thuộc vào bất kỳ thư viện (crate) bên thứ ba nào**.

KeyMouse mang lại cảm giác di chuột mượt mà như chuột vật lý nhờ động cơ mô phỏng vật lý 125 FPS với tích lũy toạ độ dưới điểm ảnh (subpixel accumulator). Đây là giải pháp hoàn hảo khi chuột hết pin, cổ tay bị mỏi do dùng chuột lâu, hoặc khi cần căn chỉnh con trỏ chính xác từng điểm ảnh mà không cần rời tay khỏi bàn phím.

### Tính năng nổi bật

- **🚀 Động cơ vật lý 125 FPS Subpixel**:
  - Tích luỹ toạ độ vi điểm (fractional subpixel) giúp chuột lướt cực êm, không bị giật khấc.
  - Đường cong gia tốc thông minh: dễ dàng tùy chỉnh Tốc độ tối thiểu, Tốc độ tối đa, Thời gian đạt tốc độ tối đa (ms) và Hệ số Turbo.
  - Hỗ trợ di chuyển chéo 8 hướng mượt mà khi bấm tổ hợp phím (như `W+D`, `S+A`).
- **🎯 Giữ chuột kéo thả (Drag & Drop) & Chọn vùng màn hình**:
  - Phím Space gửi sự kiện `MOUSEEVENTF_LEFTDOWN` / `MOUSEEVENTF_LEFTUP` chuẩn hệ thống.
  - Cho phép giữ phím Space để kéo thả icon, bôi đen văn bản, vẽ hình hoặc chọn vùng chụp màn hình (Snipping Tool, Lightshot).
  - Tách riêng chức năng Chuột phải (`X`), Cuộn lên (`Q`) và Cuộn xuống (`E`).
- **✨ Thông báo OSD Toast khử răng cưa mượt mà**:
  - Cửa sổ Win32 Layered nổi trên cùng (`WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST`).
  - Viền bo cong khử răng cưa (ClearType font + GDI alpha-blending).
  - Tùy chỉnh 4 vị trí hiển thị: `Trên - Giữa`, `Trên - Phải`, `Dưới - Giữa`, `Dưới - Phải`.
  - Hiển thị tức thì trạng thái Bật/Tắt chế độ chuột và phím tắt.
- **🎨 Giao diện Fluent Design Windows 11**:
  - Tích hợp micro-server cục bộ (`127.0.0.1:28888`) mở dưới dạng ứng dụng Edge App Mode độc lập (`--app`).
  - Hiệu ứng Mica/Acrylic Dark Mode hiện đại, sang trọng.
  - Sơ đồ chuột trực quan tương tác với các đường chỉ dẫn SVG nối trực tiếp phím gán vào từng nút chuột.
  - Cụm D-Pad hiển thị trực quan chuyển động WASD theo thời gian thực.
  - Bộ ghi phím tắt (Hotkey Recorder) trực quan, đổi phím chỉ với một lần nhấn.
- **🌐 Song ngữ Anh - Việt tiện lợi**:
  - Chuyển đổi ngôn ngữ tức thì giữa Tiếng Việt (`VI`) và Tiếng Anh (`EN`) cho toàn bộ giao diện và thông báo Toast OSD.
- **🔔 Khay hệ thống (System Tray) & Icon độ phân giải cao**:
  - Bộ biểu tượng chuột công thái học độ nét cao (16x16, 32x32, 48x48, 64x64, 256x256).
  - Icon khay hệ thống hiển thị trực quan trạng thái (Đèn LED xanh khi BẬT, xám mờ khi TẮT).
  - Menu chuột phải tiện lợi: Mở cài đặt, Bật/Tắt chế độ, Thoát ứng dụng. Double click vào icon khay để chuyển trạng thái.
- **🛡️ Không chiếm dụng phím ngoài ý muốn**:
  - Không can thiệp vào các phím mũi tên điều hướng mặc định của bàn phím.
  - Phím `Esc` chỉ đóng cửa sổ cài đặt khi đang tập trung (focus) vào ứng dụng, không làm ảnh hưởng đến các tác vụ hệ thống khác.
- **⚡ Chế độ Administrator & Điều khiển Task Manager toàn diện**:
  - Tự động phát hiện và hỗ trợ khởi động với quyền Administrator để vượt qua cơ chế bảo mật UIPI (User Interface Privilege Isolation) của Windows.
  - Đảm bảo bắt phím và di chuột liên tục trên mọi ứng dụng hệ thống đặc quyền cao như **Task Manager** (kể cả khi bật chế độ "Luôn ở trên cùng" - Always on top), Regedit hay CMD/PowerShell Admin.
  - Tích hợp nút kiểm tra và nâng quyền Administrator trực tiếp trên thanh tiêu đề của giao diện chỉ với 1 cú click.

### Phím tắt mặc định

| Thao tác | Phím mặc định | Ghi chú |
| :--- | :--- | :--- |
| **Bật / Tắt chế độ chuột** | `F8` | Phím tắt toàn cục (tự do thay đổi) |
| **Di chuyển Lên / Xuống / Trái / Phải** | `W` / `S` / `A` / `D` | Hỗ trợ đi chéo mượt mà |
| **Chuột trái / Giữ để kéo thả** | `Space` | Giữ để bôi đen, vẽ, kéo thả |
| **Chuột phải** | `X` | Mở menu ngữ cảnh |
| **Cuộn lên** | `Q` | Cuộn bánh xe lên |
| **Cuộn xuống** | `E` | Cuộn bánh xe xuống |
| **Đóng cửa sổ cài đặt** | `Esc` | Đóng bảng điều khiển khi đang mở |

### Hướng dẫn cài đặt & Biên dịch

#### Yêu cầu hệ thống
- Windows 10 hoặc Windows 11 (64-bit).
- [Bộ công cụ Rust](https://rustup.rs/) (bản 2024 trở lên).
- Trình duyệt Microsoft Edge (mặc định đã có sẵn trên Windows 10/11).

#### Biên dịch từ mã nguồn

```powershell
# 1. Clone repository về máy
git clone https://github.com/phatMT97/keymouse.git
cd keymouse

# 2. Biên dịch bản release tối ưu hoá cao nhất
cargo build --release

# 3. Chạy phần mềm
./target/release/keymouse.exe
```

File thực thi sau khi biên dịch hoàn toàn độc lập, dung lượng siêu nhẹ (~1.5 MB) và không cần cài đặt thêm bất kỳ thư viện ngoài nào.

---

### License

Distributed under the [MIT License](LICENSE).
