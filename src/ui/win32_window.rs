#[cfg(windows)]
pub mod native {
    use std::ptr::null_mut;
    use std::sync::Mutex;
    use windows_sys::Win32::Foundation::*;
    use windows_sys::Win32::Graphics::Gdi::*;
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetKeyState;
    use windows_sys::Win32::UI::WindowsAndMessaging::*;

    use crate::colors::presets;
    use crate::ui::TabManager;

    static mut APP_STATE: Option<Mutex<TabManager>> = None;

    const TAB_BAR_HEIGHT: i32 = 34;
    const FONT_HEIGHT: i32 = 18;
    const FONT_WIDTH: i32 = 9;

    pub fn run_native_window() {
        unsafe {
            let instance = GetModuleHandleW(null_mut());
            let class_name = wide_string("TermCraftWindowClass");

            let wnd_class = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(wnd_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: instance,
                hIcon: LoadIconW(0, IDI_APPLICATION),
                hCursor: LoadCursorW(0, IDC_ARROW),
                hbrBackground: 0 as HBRUSH,
                lpszMenuName: null_mut(),
                lpszClassName: class_name.as_ptr(),
            };

            RegisterClassW(&wnd_class);

            let palette = presets::iterm_default();
            APP_STATE = Some(Mutex::new(TabManager::new(100, 32, &palette)));

            let window_title = wide_string("TermCraft — iTerm2 for Windows");

            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                window_title.as_ptr(),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                960,
                640,
                0,
                0,
                instance,
                null_mut(),
            );

            if hwnd == 0 {
                eprintln!("Erreur lors de la création de la fenêtre native Win32.");
                return;
            }

            ShowWindow(hwnd, SW_SHOW);
            UpdateWindow(hwnd);

            // Timer de rafraîchissement 60 FPS pour lire les sorties du PTY Shell
            SetTimer(hwnd, 1, 16, None);

            let mut msg = std::mem::zeroed();
            while GetMessageW(&mut msg, 0, 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }

    unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match msg {
            WM_TIMER => {
                // Poll des sorties Shell de l'onglet actif
                if let Some(ref state) = APP_STATE {
                    if let Ok(mut mgr) = state.lock() {
                        let palette = presets::iterm_default();
                        let active = mgr.active_tab_mut();
                        active.poll_pty(&palette);
                    }
                }
                InvalidateRect(hwnd, null_mut(), 0);
                0
            }

            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(hwnd, &mut ps);
                draw_terminal_window(hwnd, hdc);
                EndPaint(hwnd, &ps);
                0
            }

            WM_CHAR => {
                let ch = char::from_u32(wparam as u32).unwrap_or('\0');
                if ch != '\0' {
                    if let Some(ref state) = APP_STATE {
                        if let Ok(mut mgr) = state.lock() {
                            let mut buf = [0u8; 4];
                            let s = ch.encode_utf8(&mut buf);
                            mgr.active_tab_mut().send_input(s);
                        }
                    }
                }
                0
            }

            WM_KEYDOWN => {
                // Raccourcis clavier iTerm2
                let is_ctrl = (GetKeyState(0x11) as u16 & 0x8000) != 0; // VK_CONTROL
                if is_ctrl && (wparam == 0x54) { // Ctrl + T : Nouvel Onglet
                    if let Some(ref state) = APP_STATE {
                        if let Ok(mut mgr) = state.lock() {
                            let p = presets::iterm_default();
                            mgr.new_tab(&p);
                        }
                    }
                    InvalidateRect(hwnd, null_mut(), 0);
                    return 0;
                } else if is_ctrl && (wparam == 0x57) { // Ctrl + W : Fermer Onglet
                    if let Some(ref state) = APP_STATE {
                        if let Ok(mut mgr) = state.lock() {
                            mgr.close_current_tab();
                        }
                    }
                    InvalidateRect(hwnd, null_mut(), 0);
                    return 0;
                }

                // Touches spéciales vers PTY
                let seq = match wparam as u32 {
                    0x26 => Some("\x1b[A"), // Up
                    0x28 => Some("\x1b[B"), // Down
                    0x27 => Some("\x1b[C"), // Right
                    0x25 => Some("\x1b[D"), // Left
                    0x24 => Some("\x1b[H"), // Home
                    0x23 => Some("\x1b[F"), // End
                    0x2E => Some("\x1b[3~"), // Delete
                    _ => None,
                };

                if let Some(s) = seq {
                    if let Some(ref state) = APP_STATE {
                        if let Ok(mut mgr) = state.lock() {
                            mgr.active_tab_mut().send_input(s);
                        }
                    }
                }
                0
            }

            WM_LBUTTONDOWN => {
                let x = (lparam & 0xFFFF) as i32;
                let y = ((lparam >> 16) & 0xFFFF) as i32;

                if y < TAB_BAR_HEIGHT {
                    // Clic sur un onglet
                    let clicked_tab = (x / 140) as usize;
                    if let Some(ref state) = APP_STATE {
                        if let Ok(mut mgr) = state.lock() {
                            if clicked_tab < mgr.tabs.len() {
                                mgr.switch_tab(clicked_tab);
                            } else if x >= (mgr.tabs.len() as i32 * 140) && x <= (mgr.tabs.len() as i32 * 140 + 40) {
                                // Clic sur [+]
                                let p = presets::iterm_default();
                                mgr.new_tab(&p);
                            }
                        }
                    }
                    InvalidateRect(hwnd, null_mut(), 0);
                }
                0
            }

            WM_DESTROY => {
                PostQuitMessage(0);
                0
            }

            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }

    unsafe fn draw_terminal_window(hwnd: HWND, hdc: HDC) {
        let mut rect: RECT = std::mem::zeroed();
        GetClientRect(hwnd, &mut rect);
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;

        // Double-buffer GDI (Backbuffer) pour éviter tout scintillement
        let mem_dc = CreateCompatibleDC(hdc);
        let mem_bitmap = CreateCompatibleBitmap(hdc, width, height);
        let old_bitmap = SelectObject(mem_dc, mem_bitmap);

        // 1. Fond général iTerm2 (Dark #181818)
        let bg_brush = CreateSolidBrush(0x00181818);
        FillRect(mem_dc, &rect, bg_brush);
        DeleteObject(bg_brush);

        // 2. Barre d'onglets supérieure
        let tab_bar_rect = RECT {
            left: 0,
            top: 0,
            right: width,
            bottom: TAB_BAR_HEIGHT,
        };
        let tab_bar_brush = CreateSolidBrush(0x00121212);
        FillRect(mem_dc, &tab_bar_rect, tab_bar_brush);
        DeleteObject(tab_bar_brush);

        // Police pour les onglets
        let tab_font_name = wide_string("Segoe UI");
        let tab_font = CreateFontW(
            15, 0, 0, 0, 500, 0, 0, 0, DEFAULT_CHARSET as u32,
            OUT_DEFAULT_PRECIS as u32, CLIP_DEFAULT_PRECIS as u32, CLEARTYPE_QUALITY as u32,
            (DEFAULT_PITCH | FF_DONTCARE) as u32, tab_font_name.as_ptr(),
        );
        let old_font = SelectObject(mem_dc, tab_font);
        SetBkMode(mem_dc, TRANSPARENT as i32);

        if let Some(ref state) = APP_STATE {
            if let Ok(mgr) = state.lock() {
                // Dessin des onglets
                for (i, tab) in mgr.tabs.iter().enumerate() {
                    let is_active = i == mgr.active_tab_index;
                    let tab_x = (i as i32) * 140;
                    let tab_rect = RECT {
                        left: tab_x,
                        top: 2,
                        right: tab_x + 138,
                        bottom: TAB_BAR_HEIGHT,
                    };

                    let tab_bg = if is_active {
                        CreateSolidBrush(0x00222222)
                    } else {
                        CreateSolidBrush(0x00161616)
                    };
                    FillRect(mem_dc, &tab_rect, tab_bg);
                    DeleteObject(tab_bg);

                    let text_color = if is_active { 0x00FFFFFF } else { 0x00888888 };
                    SetTextColor(mem_dc, text_color);
                    let tab_title = wide_string(&format!("  {}  ", tab.title));
                    TextOutW(mem_dc, tab_x + 10, 8, tab_title.as_ptr(), tab_title.len() as i32 - 1);
                }

                // Bouton [+]
                let btn_x = mgr.tabs.len() as i32 * 140 + 6;
                SetTextColor(mem_dc, 0x0000AAFF);
                let plus_str = wide_string("+");
                TextOutW(mem_dc, btn_x, 8, plus_str.as_ptr(), 1);

                // 3. Rendu de la grille du terminal actif (Police Monospace)
                SelectObject(mem_dc, old_font);
                DeleteObject(tab_font);

                let mono_font_name = wide_string("Consolas");
                let term_font = CreateFontW(
                    FONT_HEIGHT, FONT_WIDTH, 0, 0, 400, 0, 0, 0, DEFAULT_CHARSET as u32,
                    OUT_DEFAULT_PRECIS as u32, CLIP_DEFAULT_PRECIS as u32, CLEARTYPE_QUALITY as u32,
                    (FIXED_PITCH | FF_MODERN) as u32, mono_font_name.as_ptr(),
                );
                SelectObject(mem_dc, term_font);

                let active_tab = mgr.active_tab();
                let screen = &active_tab.screen;

                let start_y = TAB_BAR_HEIGHT + 6;
                let start_x = 10;

                for row in 0..screen.rows {
                    let y_pos = start_y + (row as i32 * FONT_HEIGHT);
                    if y_pos + FONT_HEIGHT > height {
                        break;
                    }

                    for col in 0..screen.cols {
                        let x_pos = start_x + (col as i32 * FONT_WIDTH);
                        if x_pos + FONT_WIDTH > width {
                            break;
                        }

                        let cell = screen.grid[row][col];
                        let is_cursor = screen.cursor_visible && row == screen.cursor_y && col == screen.cursor_x;

                        // BGR format pour Win32 GDI
                        let fg_color = ((cell.fg.b as u32) << 16) | ((cell.fg.g as u32) << 8) | (cell.fg.r as u32);
                        let bg_color = ((cell.bg.b as u32) << 16) | ((cell.bg.g as u32) << 8) | (cell.bg.r as u32);

                        if cell.bg != screen.default_bg {
                            SetBkMode(mem_dc, 2); // OPAQUE
                            SetBkColor(mem_dc, bg_color);
                        } else {
                            SetBkMode(mem_dc, TRANSPARENT as i32);
                        }

                        if is_cursor {
                            let cur_rect = RECT {
                                left: x_pos,
                                top: y_pos,
                                right: x_pos + FONT_WIDTH,
                                bottom: y_pos + FONT_HEIGHT,
                            };
                            let cur_brush = CreateSolidBrush(0x00CCCCCC);
                            FillRect(mem_dc, &cur_rect, cur_brush);
                            DeleteObject(cur_brush);
                            SetTextColor(mem_dc, 0x00000000);
                        } else {
                            SetTextColor(mem_dc, fg_color);
                        }

                        if cell.c != ' ' {
                            let mut buf = [0u16; 2];
                            cell.c.encode_utf16(&mut buf);
                            TextOutW(mem_dc, x_pos, y_pos, buf.as_ptr(), 1);
                        }
                    }
                }

                SelectObject(mem_dc, old_font);
                DeleteObject(term_font);
            }
        }

        // Transfert du buffer en mémoire vers l'écran en un bloc (BitBlt sans aucun scintillement)
        BitBlt(hdc, 0, 0, width, height, mem_dc, 0, 0, SRCCOPY);

        SelectObject(mem_dc, old_bitmap);
        DeleteObject(mem_bitmap);
        DeleteDC(mem_dc);
    }

    fn wide_string(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }
}
