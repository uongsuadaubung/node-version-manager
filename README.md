# ⚡ Node Version Manager (Rust TUI Edition)

Một công cụ quản lý phiên bản Node.js (NVM) siêu nhanh, cực nhẹ (~2.59 MB standalone), giao diện dòng lệnh hiện đại (**TUI - Terminal User Interface**) điều khiển hoàn toàn bằng phím, hỗ trợ đa nền tảng (Windows & Linux).

```text
 ┌── ⚡ NVM-RS ─── ● Đang dùng: v22.14.0 ──────────────┐ ┌─── 📁 Lưu tại: C:\Users\user\.nvm-rust   [L: VI] ─────┐
 │                                                     │ │                                                     │
 └─────────────────────────────────────────────────────┘ └─────────────────────────────────────────────────────┘
 ┌── 1. Tất cả (800+) | 2. Bản LTS (24) | 3. Đã cài đặt (3) ─┐ ┌── 🔍 Tìm kiếm (vd: 22, lts, iron)... ──────────┐
 │                                                            │ │                                                 │
 └────────────────────────────────────────────────────────────┘ └─────────────────────────────────────────────────┘
 ┌── Node.js Versions (1/24) ─────────────────────────────────┐ ┌── Chi tiết phiên bản ───────────────────────────┐
 │   Phiên bản     Phân loại     Trạng thái    Modules  Ngày  │ │ Phiên bản: v22.14.0                           │
 │ ▶ v22.14.0      LTS (Jod)     ● Đang dùng   [Shared] 2025  │ │ LTS: Có (Jod)                                 │
 │   v20.18.3      LTS (Iron)    ✓ Đã tải      [Isolate]2025  │ │ Trạng thái: ● Đang dùng                       │
 │   v18.20.7      LTS (Hydrogen)○ Chưa tải    -        2025  │ │ Đường dẫn cài đặt: ~/.nvm-rust/versions/...  │
 └────────────────────────────────────────────────────────────┘ └─────────────────────────────────────────────────┘
 ┌── 💬 Sẵn sàng... ───────────────────────────────────────────────────────────────────────────────────────────┐
 └─────────────────────────────────────────────────────────────────────────────────────────────────────────────┘
  ⚡ Thao tác:  [u]:Bỏ dùng (gỡ PATH)   [d]:Gỡ bỏ   [s]:Đổi Shared modules
  🧭 Điều khiển:  [↑/↓]:Chọn  [1-3]:Tab  [/]:Tìm kiếm  [r]:Tải lại  [m]:Nơi lưu  [l]:Ngôn ngữ  [?]:Trợ giúp  [q]:Thoát
```

## ✨ Ưu điểm vượt trội

- **Siêu nhẹ & Độc lập:** File `.exe` chỉ **~2.59 MB**, không cần WebView, không cần Node.js, không phụ thuộc PowerShell script.
- **Khởi động tức thì:** Khởi động dưới **20ms**, tiêu thụ chỉ ~10MB RAM.
- **Điều khiển phím tốc độ cao:** Tương tự `lazygit`, `btop`, `fzf` — tìm kiếm, cài đặt, chuyển đổi chỉ với 1-2 phím gõ.
- **Gauge thanh tiến trình trực tiếp:** Tải trực tiếp qua HTTP streaming với thanh % dung lượng (MB) vẽ mượt mà ngay trên terminal.
- **Quản lý PATH bản địa (Native):**
  - **Windows:** Cập nhật Registry `HKCU\Environment\Path` và broadcast `WM_SETTINGCHANGE` giúp các terminal nhận diện ngay lập tức.
  - **Linux:** Symlink farm tại `~/.local/bin`, sạch sẽ, không ghi đè `.bashrc`.
- **Global NPM Modules thông minh:** Tùy chọn chia sẻ thư mục global modules (`prefix=~/.nvm-rust/modules`) hoặc cô lập riêng từng version (nhấn phím `s` để đổi).
- **Lưu trữ linh hoạt:** Đổi thư mục lưu trữ bất kỳ lúc nào bằng phím `m` với giao diện modal hỗ trợ paste clipboard (`Ctrl+V`) và xóa nhanh (`Ctrl+U`).
- **Đa ngôn ngữ (i18n):** Hỗ trợ đầy đủ Tiếng Việt và English (nhấn phím `l` để chuyển đổi tức thì).

## ⌨️ Bảng phím tắt (Keybindings)

| Phím | Chức năng |
| :--- | :--- |
| `↑` / `k`, `↓` / `j` | Di chuyển lên / xuống danh sách phiên bản |
| `PgUp` / `PgDn` | Cuộn trang nhanh (10 phiên bản) |
| `Home` / `End` | Nhảy về đầu danh sách / cuối danh sách |
| `1`, `2`, `3` | Chuyển Tabs: **[1] Tất cả**, **[2] Bản LTS**, **[3] Đã cài đặt** |
| `Tab` | Chuyển tuần tự giữa các Tab |
| `Enter` | **Sử dụng phiên bản:** Tự động kích hoạt PATH (tự tải và cài nếu chưa có) |
| `i` | Tải và cài đặt phiên bản được chọn |
| `d` / `Delete` | Gỡ bỏ phiên bản đã cài đặt (có modal xác nhận an toàn `y/n`) |
| `s` | Bật / Tắt chế độ dùng chung Global npm modules (`Shared / Isolated`) |
| `u` | Gỡ Node khỏi biến môi trường PATH (deactivate / unuse) |
| `m` | Đổi đường dẫn thư mục lưu trữ (Storage directory modal) |
| `/` | Bật chế độ tìm kiếm nhanh (gõ số hiệu `22`, `v20` hoặc codename `iron`, `jod`) |
| `Esc` | Xóa tìm kiếm / Đóng các popup modal |
| `r` | Làm mới danh sách phiên bản trực tiếp từ nodejs.org |
| `l` | Chuyển đổi ngôn ngữ hiển thị (Tiếng Việt / English) |
| `?` / `h` | Mở bảng trợ giúp phím tắt toàn màn hình |
| `q` / `Ctrl+C` | Thoát ứng dụng |

## 🛠️ Cài đặt & Biên dịch

### Chạy trực tiếp bản Release
File thực thi nằm tại:
```powershell
.\target\release\node-version-manager.exe
```

### Biên dịch từ mã nguồn (Rust)
```powershell
# Chế độ phát triển (Debug)
cargo run

# Chế độ phát hành siêu tối ưu dung lượng (Release ~3.2 MB)
cargo build --release
```

## 🏗️ Công nghệ cốt lõi

- **Terminal UI:** [ratatui](https://github.com/ratatui/ratatui) + [crossterm](https://github.com/crossterm-rs/crossterm) (Hỗ trợ Unicode, UTF-8, màu ANSI/RGB, raw mode, mouse & keyboard events).
- **HTTP Client:** [reqwest](https://github.com/seanmonstar/reqwest) (Non-blocking streaming download).
- **Giải nén tốc độ cao:** [zip-rs](https://github.com/zip-rs/zip2), [tar](https://github.com/alexcrichton/tar-rs), [flate2](https://github.com/rust-lang/flate2-rs).
- **Windows Registry:** [winreg](https://github.com/gentoo90/winreg-rs) + Windows API (`SendMessageTimeoutW` with `HWND_BROADCAST`).
- **I18n:** Bản địa hóa qua TOML (`locales/vi.toml`, `locales/en.toml`).

---
*Phát triển với ❤️ bằng Rust*
