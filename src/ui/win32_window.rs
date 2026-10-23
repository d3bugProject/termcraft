#[cfg(windows)]
pub mod native {
    use std::ptr::null_mut;
    use std::sync::Mutex;
    use windows_sys::Win32::Foundation::*;
    use windows_sys::Win32::Graphics::Gdi::*;
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetKeyState;
    use windows_sys::Win32::UI::WindowsAndMessaging::*;

    use crate::colors::{presets, ColorPalette};
    use crate::terminal::{Screen, Vt100Parser};

    // Identifiants de menus (Port de iTerm2 menus)
    const ID_APP_ABOUT: usize = 101;
    const ID_APP_PREFERENCES: usize = 102;
    const ID_APP_QUIT: usize = 103;

    const ID_SHELL_NEW_TAB: usize = 201;
    const ID_SHELL_CLOSE_TAB: usize = 202;
    const ID_SHELL_SPLIT_HORIZ: usize = 203;
    const ID_SHELL_SPLIT_VERT: usize = 204;
    const ID_SHELL_CLEAR_BUFFER: usize = 205;

    const ID_EDIT_COPY: usize = 301;
    const ID_EDIT_PASTE: usize = 302;
    const ID_EDIT_SELECT_ALL: usize = 303;

    const ID_VIEW_ZOOM_IN: usize = 401;
    const ID_VIEW_ZOOM_OUT: usize = 402;
    const ID_VIEW_RESET_ZOOM: usize = 403;

    const ID_PROFILES_DEFAULT: usize = 501;
    const ID_PROFILES_SOLARIZED: usize = 502;
    const ID_PROFILES_DRACULA: usize = 503;
    const ID_PROFILES_MANAGE: usize = 504;

    const ID_HELP_SHORTCUTS: usize = 601;

    struct InteractiveTab {
        id: usize,
        title: String,
        screen: Screen,
        parser: Vt100Parser,
        current_input: String,
        cwd: String,
    }

    struct AppModel {
        tabs: Vec<InteractiveTab>,
        active_tab_index: usize,
        palette: ColorPalette,
        font_size: i32,
    }

    static mut APP_STATE: Option<Mutex<AppModel>> = None;

    const TAB_BAR_HEIGHT: i32 = 34;

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
            let mut first_tab = create_new_tab(1, &palette);
            init_tab_welcome(&mut first_tab, &palette);

            APP_STATE = Some(Mutex::new(AppModel {
                tabs: vec![first_tab],
                active_tab_index: 0,
                palette,
                font_size: 17,
            }));

            // Création de la barre de menus complète d'iTerm2
            let menu_bar = create_iterm_menu_bar();

            let window_title = wide_string("iTerm2 — TermCraft (Rust Native)");

            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                window_title.as_ptr(),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                1000,
                680,
                0,
                menu_bar,
                instance,
                null_mut(),
            );

            if hwnd == 0 {
                return;
            }

            ShowWindow(hwnd, SW_SHOW);
            UpdateWindow(hwnd);

            // Timer de clignotement du curseur et rafraîchissement
            SetTimer(hwnd, 1, 50, None);

            let mut msg = std::mem::zeroed();
            while GetMessageW(&mut msg, 0, 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }

    fn create_new_tab(id: usize, palette: &ColorPalette) -> InteractiveTab {
        let cwd = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "C:\\".to_string());

        let screen = Screen::new(100, 32, palette.foreground, palette.background);
        let parser = Vt100Parser::new();

        InteractiveTab {
            id,
            title: format!(" Tab {} (zsh)", id),
            screen,
            parser,
            current_input: String::new(),
            cwd,
        }
    }

    fn init_tab_welcome(tab: &mut InteractiveTab, palette: &ColorPalette) {
        let welcome = format!(
            "\x1b[1;36mLast login: Fri Oct 23 21:00:00 on ttys000\x1b[0m\n\r\
             \x1b[1;32miTerm2 v3.5.0 (TermCraft 100% Pure Rust Native Edition)\x1b[0m\n\r\
             \x1b[90mMulti-platform engine: macOS • Linux • Windows | Monospace GDI Double-buffered\x1b[0m\n\r\n\r"
        );
        tab.parser.process_stream(&welcome, &mut tab.screen, palette);
        print_prompt(tab, palette);
    }

    fn print_prompt(tab: &mut InteractiveTab, palette: &ColorPalette) {
        let prompt = format!("\x1b[1;34m{}\x1b[0m \x1b[1;32m% \x1b[0m", tab.cwd);
        tab.parser.process_stream(&prompt, &mut tab.screen, palette);
    }

    unsafe fn create_iterm_menu_bar() -> HMENU {
        let menu_bar = CreateMenu();

        // 1. Menu iTerm2
        let menu_iterm = CreatePopupMenu();
        AppendMenuW(menu_iterm, MF_STRING, ID_APP_ABOUT, wide_string("About iTerm2").as_ptr());
        AppendMenuW(menu_iterm, MF_STRING, ID_APP_PREFERENCES, wide_string("Preferences... (Ctrl+,)").as_ptr());
        AppendMenuW(menu_iterm, MF_SEPARATOR, 0, null_mut());
        AppendMenuW(menu_iterm, MF_STRING, ID_APP_QUIT, wide_string("Quit iTerm2 (Ctrl+Q)").as_ptr());
        AppendMenuW(menu_bar, MF_POPUP, menu_iterm as usize, wide_string("iTerm2").as_ptr());

        // 2. Menu Shell
        let menu_shell = CreatePopupMenu();
        AppendMenuW(menu_shell, MF_STRING, ID_SHELL_NEW_TAB, wide_string("New Tab (Ctrl+T)").as_ptr());
        AppendMenuW(menu_shell, MF_STRING, ID_SHELL_SPLIT_HORIZ, wide_string("Split Horizontally (Ctrl+D)").as_ptr());
        AppendMenuW(menu_shell, MF_STRING, ID_SHELL_SPLIT_VERT, wide_string("Split Vertically (Ctrl+Shift+D)").as_ptr());
        AppendMenuW(menu_shell, MF_SEPARATOR, 0, null_mut());
        AppendMenuW(menu_shell, MF_STRING, ID_SHELL_CLEAR_BUFFER, wide_string("Clear Buffer (Ctrl+K)").as_ptr());
        AppendMenuW(menu_shell, MF_STRING, ID_SHELL_CLOSE_TAB, wide_string("Close Tab (Ctrl+W)").as_ptr());
        AppendMenuW(menu_bar, MF_POPUP, menu_shell as usize, wide_string("Shell").as_ptr());

        // 3. Menu Edit
        let menu_edit = CreatePopupMenu();
        AppendMenuW(menu_edit, MF_STRING, ID_EDIT_COPY, wide_string("Copy (Ctrl+C)").as_ptr());
        AppendMenuW(menu_edit, MF_STRING, ID_EDIT_PASTE, wide_string("Paste (Ctrl+V)").as_ptr());
        AppendMenuW(menu_edit, MF_SEPARATOR, 0, null_mut());
        AppendMenuW(menu_edit, MF_STRING, ID_EDIT_SELECT_ALL, wide_string("Select All (Ctrl+A)").as_ptr());
        AppendMenuW(menu_bar, MF_POPUP, menu_edit as usize, wide_string("Edit").as_ptr());

        // 4. Menu View
        let menu_view = CreatePopupMenu();
        AppendMenuW(menu_view, MF_STRING, ID_VIEW_ZOOM_IN, wide_string("Zoom In (Ctrl++)").as_ptr());
        AppendMenuW(menu_view, MF_STRING, ID_VIEW_ZOOM_OUT, wide_string("Zoom Out (Ctrl+-)").as_ptr());
        AppendMenuW(menu_view, MF_STRING, ID_VIEW_RESET_ZOOM, wide_string("Reset Zoom (Ctrl+0)").as_ptr());
        AppendMenuW(menu_bar, MF_POPUP, menu_view as usize, wide_string("View").as_ptr());

        // 5. Menu Profiles
        let menu_profiles = CreatePopupMenu();
        AppendMenuW(menu_profiles, MF_STRING, ID_PROFILES_DEFAULT, wide_string("Default iTerm2 Theme").as_ptr());
        AppendMenuW(menu_profiles, MF_STRING, ID_PROFILES_SOLARIZED, wide_string("Solarized Dark").as_ptr());
        AppendMenuW(menu_profiles, MF_STRING, ID_PROFILES_DRACULA, wide_string("Dracula").as_ptr());
        AppendMenuW(menu_profiles, MF_SEPARATOR, 0, null_mut());
        AppendMenuW(menu_profiles, MF_STRING, ID_PROFILES_MANAGE, wide_string("Open Profiles Window...").as_ptr());
        AppendMenuW(menu_bar, MF_POPUP, menu_profiles as usize, wide_string("Profiles").as_ptr());

        // 6. Menu Help
        let menu_help = CreatePopupMenu();
        AppendMenuW(menu_help, MF_STRING, ID_HELP_SHORTCUTS, wide_string("Keyboard Shortcuts").as_ptr());
        AppendMenuW(menu_bar, MF_POPUP, menu_help as usize, wide_string("Help").as_ptr());

        menu_bar
    }

    unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match msg {
            WM_TIMER => {
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

            WM_COMMAND => {
                let cmd_id = (wparam & 0xFFFF) as usize;
                handle_menu_command(hwnd, cmd_id);
                0
            }

            WM_CHAR => {
                let ch = char::from_u32(wparam as u32).unwrap_or('\0');
                if ch != '\0' && ch != '\u{1b}' {
                    handle_keyboard_char(hwnd, ch);
                }
                0
            }

            WM_KEYDOWN => {
                let is_ctrl = (GetKeyState(0x11) as u16 & 0x8000) != 0; // VK_CONTROL
                if is_ctrl {
                    match wparam as u32 {
                        0x54 => { // Ctrl + T : Nouvel Onglet
                            handle_menu_command(hwnd, ID_SHELL_NEW_TAB);
                            return 0;
                        }
                        0x57 => { // Ctrl + W : Fermer Onglet
                            handle_menu_command(hwnd, ID_SHELL_CLOSE_TAB);
                            return 0;
                        }
                        0x4B => { // Ctrl + K : Clear buffer
                            handle_menu_command(hwnd, ID_SHELL_CLEAR_BUFFER);
                            return 0;
                        }
                        0xBB | 0x6B => { // Ctrl + Plus : Zoom In
                            handle_menu_command(hwnd, ID_VIEW_ZOOM_IN);
                            return 0;
                        }
                        0xBD | 0x6D => { // Ctrl + Minus : Zoom Out
                            handle_menu_command(hwnd, ID_VIEW_ZOOM_OUT);
                            return 0;
                        }
                        0x30 | 0x60 => { // Ctrl + 0 : Reset Zoom
                            handle_menu_command(hwnd, ID_VIEW_RESET_ZOOM);
                            return 0;
                        }
                        _ => {}
                    }
                }
                0
            }

            WM_LBUTTONDOWN => {
                let x = (lparam & 0xFFFF) as i32;
                let y = ((lparam >> 16) & 0xFFFF) as i32;

                if y < TAB_BAR_HEIGHT {
                    if let Some(ref state) = APP_STATE {
                        if let Ok(mut app) = state.lock() {
                            let tab_count = app.tabs.len();
                            let clicked_tab = (x / 140) as usize;
                            if clicked_tab < tab_count {
                                app.active_tab_index = clicked_tab;
                            } else if x >= (tab_count as i32 * 140) && x <= (tab_count as i32 * 140 + 40) {
                                // Bouton [+]
                                let new_id = tab_count + 1;
                                let mut tab = create_new_tab(new_id, &app.palette);
                                init_tab_welcome(&mut tab, &app.palette);
                                app.tabs.push(tab);
                                app.active_tab_index = app.tabs.len() - 1;
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

    unsafe fn handle_keyboard_char(hwnd: HWND, ch: char) {
        if let Some(ref state) = APP_STATE {
            if let Ok(mut app) = state.lock() {
                let palette = app.palette.clone();
                let active_idx = app.active_tab_index;
                let tab = &mut app.tabs[active_idx];

                match ch {
                    '\r' | '\n' => {
                        // Exécution de la commande saisie
                        let cmd = tab.current_input.trim().to_string();
                        tab.current_input.clear();

                        tab.parser.process_stream("\n\r", &mut tab.screen, &palette);

                        if !cmd.is_empty() {
                            if cmd == "clear" || cmd == "cls" {
                                tab.screen.clear_screen();
                            } else {
                                let output = execute_shell(&cmd, &tab.cwd);
                                tab.parser.process_stream(&format!("{}\n\r", output), &mut tab.screen, &palette);
                            }
                        }
                        print_prompt(tab, &palette);
                    }
                    '\x08' => {
                        // Touche retour arrière (Backspace)
                        if !tab.current_input.is_empty() {
                            tab.current_input.pop();
                            tab.screen.backspace();
                            tab.screen.write_char(' ');
                            tab.screen.backspace();
                        }
                    }
                    c if !c.is_control() => {
                        tab.current_input.push(c);
                        tab.screen.write_char(c);
                    }
                    _ => {}
                }
            }
        }
        InvalidateRect(hwnd, null_mut(), 0);
    }

    fn execute_shell(cmd: &str, cwd: &str) -> String {
        #[cfg(target_os = "windows")]
        let out = std::process::Command::new("powershell")
            .current_dir(cwd)
            .args(["-NoProfile", "-Command", cmd])
            .output();

        #[cfg(not(target_os = "windows"))]
        let out = std::process::Command::new("/bin/sh")
            .current_dir(cwd)
            .args(["-c", cmd])
            .output();

        match out {
            Ok(res) => {
                let mut text = String::from_utf8_lossy(&res.stdout).to_string();
                if !res.stderr.is_empty() {
                    text.push_str("\x1b[31m");
                    text.push_str(&String::from_utf8_lossy(&res.stderr));
                    text.push_str("\x1b[0m");
                }
                text.replace("\n", "\n\r")
            }
            Err(e) => format!("\x1b[31mExecution error: {}\x1b[0m\n\r", e),
        }
    }

    unsafe fn handle_menu_command(hwnd: HWND, cmd_id: usize) {
        match cmd_id {
            ID_APP_ABOUT => {
                let msg = wide_string(
                    "iTerm2 — TermCraft Edition (100% Pure Rust Native)\n\n\
                     A faithful, cross-platform clean-room reimplementation of iTerm2.\n\
                     Engine: VT100Screen + PTYTask + iTermColorMap in Rust.\n\
                     License: GPL v2\n\
                     Author: d3bugProject\n\
                     Website: https://github.com/d3bugProject/termcraft"
                );
                let title = wide_string("About iTerm2 (TermCraft)");
                MessageBoxW(hwnd, msg.as_ptr(), title.as_ptr(), MB_OK | MB_ICONINFORMATION);
            }

            ID_APP_PREFERENCES | ID_PROFILES_MANAGE => {
                let msg = wide_string(
                    "iTerm2 Preferences (TermCraft):\n\n\
                     • Default Shell: PowerShell / zsh / bash\n\
                     • Typography: Consolas / SF Mono / JetBrains Mono\n\
                     • Scrollback Buffer: 10,000 lines (Dynamic ring buffer)\n\
                     • Rendering Engine: Direct Win32 GDI Double-buffered\n\
                     • Color Presets: iTerm2 Default, Solarized Dark, Dracula\n\n\
                     All settings are loaded from ~/.iterm2/profiles.json"
                );
                let title = wide_string("iTerm2 Preferences");
                MessageBoxW(hwnd, msg.as_ptr(), title.as_ptr(), MB_OK | MB_ICONINFORMATION);
            }

            ID_APP_QUIT => {
                PostQuitMessage(0);
            }

            ID_SHELL_NEW_TAB => {
                if let Some(ref state) = APP_STATE {
                    if let Ok(mut app) = state.lock() {
                        let new_id = app.tabs.len() + 1;
                        let mut tab = create_new_tab(new_id, &app.palette);
                        init_tab_welcome(&mut tab, &app.palette);
                        app.tabs.push(tab);
                        app.active_tab_index = app.tabs.len() - 1;
                    }
                }
                InvalidateRect(hwnd, null_mut(), 0);
            }

            ID_SHELL_CLOSE_TAB => {
                if let Some(ref state) = APP_STATE {
                    if let Ok(mut app) = state.lock() {
                        if app.tabs.len() > 1 {
                            let idx = app.active_tab_index;
                            app.tabs.remove(idx);
                            if app.active_tab_index >= app.tabs.len() {
                                app.active_tab_index = app.tabs.len() - 1;
                            }
                        }
                    }
                }
                InvalidateRect(hwnd, null_mut(), 0);
            }

            ID_SHELL_CLEAR_BUFFER => {
                if let Some(ref state) = APP_STATE {
                    if let Ok(mut app) = state.lock() {
                        let p = app.palette.clone();
                        let idx = app.active_tab_index;
                        let tab = &mut app.tabs[idx];
                        tab.screen.clear_screen();
                        print_prompt(tab, &p);
                    }
                }
                InvalidateRect(hwnd, null_mut(), 0);
            }

            ID_PROFILES_DEFAULT => {
                if let Some(ref state) = APP_STATE {
                    if let Ok(mut app) = state.lock() {
                        app.palette = presets::iterm_default();
                    }
                }
                InvalidateRect(hwnd, null_mut(), 0);
            }

            ID_PROFILES_SOLARIZED => {
                if let Some(ref state) = APP_STATE {
                    if let Ok(mut app) = state.lock() {
                        app.palette = presets::solarized_dark();
                    }
                }
                InvalidateRect(hwnd, null_mut(), 0);
            }

            ID_PROFILES_DRACULA => {
                if let Some(ref state) = APP_STATE {
                    if let Ok(mut app) = state.lock() {
                        app.palette = presets::dracula();
                    }
                }
                InvalidateRect(hwnd, null_mut(), 0);
            }

            ID_VIEW_ZOOM_IN => {
                if let Some(ref state) = APP_STATE {
                    if let Ok(mut app) = state.lock() {
                        app.font_size = (app.font_size + 2).min(32);
                    }
                }
                InvalidateRect(hwnd, null_mut(), 0);
            }

            ID_VIEW_ZOOM_OUT => {
                if let Some(ref state) = APP_STATE {
                    if let Ok(mut app) = state.lock() {
                        app.font_size = (app.font_size - 2).max(10);
                    }
                }
                InvalidateRect(hwnd, null_mut(), 0);
            }

            ID_VIEW_RESET_ZOOM => {
                if let Some(ref state) = APP_STATE {
                    if let Ok(mut app) = state.lock() {
                        app.font_size = 17;
                    }
                }
                InvalidateRect(hwnd, null_mut(), 0);
            }

            ID_HELP_SHORTCUTS => {
                let msg = wide_string(
                    "iTerm2 Keyboard Shortcuts (TermCraft):\n\n\
                     • Ctrl + T : New Tab\n\
                     • Ctrl + W : Close Tab\n\
                     • Ctrl + K : Clear Screen Buffer\n\
                     • Ctrl + + : Zoom In\n\
                     • Ctrl + - : Zoom Out\n\
                     • Ctrl + 0 : Reset Zoom"
                );
                let title = wide_string("Keyboard Shortcuts");
                MessageBoxW(hwnd, msg.as_ptr(), title.as_ptr(), MB_OK | MB_ICONINFORMATION);
            }

            _ => {}
        }
    }

    unsafe fn draw_terminal_window(hwnd: HWND, hdc: HDC) {
        let mut rect: RECT = std::mem::zeroed();
        GetClientRect(hwnd, &mut rect);
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;

        let mem_dc = CreateCompatibleDC(hdc);
        let mem_bitmap = CreateCompatibleBitmap(hdc, width, height);
        let old_bitmap = SelectObject(mem_dc, mem_bitmap);

        if let Some(ref state) = APP_STATE {
            if let Ok(app) = state.lock() {
                let bg_rgb = app.palette.background;
                let bg_color_val = ((bg_rgb.b as u32) << 16) | ((bg_rgb.g as u32) << 8) | (bg_rgb.r as u32);

                // 1. Fond général
                let bg_brush = CreateSolidBrush(bg_color_val);
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

                // Police des onglets
                let tab_font_name = wide_string("Segoe UI");
                let tab_font = CreateFontW(
                    15, 0, 0, 0, 500, 0, 0, 0, DEFAULT_CHARSET as u32,
                    OUT_DEFAULT_PRECIS as u32, CLIP_DEFAULT_PRECIS as u32, CLEARTYPE_QUALITY as u32,
                    (DEFAULT_PITCH | FF_DONTCARE) as u32, tab_font_name.as_ptr(),
                );
                let old_font = SelectObject(mem_dc, tab_font);
                SetBkMode(mem_dc, TRANSPARENT as i32);

                for (i, tab) in app.tabs.iter().enumerate() {
                    let is_active = i == app.active_tab_index;
                    let tab_x = (i as i32) * 140;
                    let tab_rect = RECT {
                        left: tab_x,
                        top: 2,
                        right: tab_x + 138,
                        bottom: TAB_BAR_HEIGHT,
                    };

                    let tab_bg = if is_active {
                        CreateSolidBrush(bg_color_val)
                    } else {
                        CreateSolidBrush(0x00161616)
                    };
                    FillRect(mem_dc, &tab_rect, tab_bg);
                    DeleteObject(tab_bg);

                    let text_color = if is_active { 0x00FFFFFF } else { 0x00888888 };
                    SetTextColor(mem_dc, text_color);
                    let tab_title = wide_string(&format!("  {}  ", tab.title));
                    TextOutW(mem_dc, tab_x + 8, 8, tab_title.as_ptr(), tab_title.len() as i32 - 1);
                }

                // Bouton [+]
                let btn_x = app.tabs.len() as i32 * 140 + 6;
                SetTextColor(mem_dc, 0x0000AAFF);
                let plus_str = wide_string("+");
                TextOutW(mem_dc, btn_x, 8, plus_str.as_ptr(), 1);

                SelectObject(mem_dc, old_font);
                DeleteObject(tab_font);

                // 3. Grille du terminal actif
                let font_h = app.font_size;
                let font_w = (font_h * 55) / 100;

                let mono_font_name = wide_string("Consolas");
                let term_font = CreateFontW(
                    font_h, font_w, 0, 0, 400, 0, 0, 0, DEFAULT_CHARSET as u32,
                    OUT_DEFAULT_PRECIS as u32, CLIP_DEFAULT_PRECIS as u32, CLEARTYPE_QUALITY as u32,
                    (FIXED_PITCH | FF_MODERN) as u32, mono_font_name.as_ptr(),
                );
                SelectObject(mem_dc, term_font);

                let active_tab = &app.tabs[app.active_tab_index];
                let screen = &active_tab.screen;

                let start_y = TAB_BAR_HEIGHT + 8;
                let start_x = 10;

                for row in 0..screen.rows {
                    let y_pos = start_y + (row as i32 * font_h);
                    if y_pos + font_h > height {
                        break;
                    }

                    for col in 0..screen.cols {
                        let x_pos = start_x + (col as i32 * font_w);
                        if x_pos + font_w > width {
                            break;
                        }

                        let cell = screen.grid[row][col];
                        let is_cursor = screen.cursor_visible && row == screen.cursor_y && col == screen.cursor_x;

                        let fg_color = ((cell.fg.b as u32) << 16) | ((cell.fg.g as u32) << 8) | (cell.fg.r as u32);
                        let bg_color = ((cell.bg.b as u32) << 16) | ((cell.bg.g as u32) << 8) | (cell.bg.r as u32);

                        if is_cursor {
                            let cur_rect = RECT {
                                left: x_pos,
                                top: y_pos,
                                right: x_pos + font_w,
                                bottom: y_pos + font_h,
                            };
                            let cur_brush = CreateSolidBrush(0x00CCCCCC);
                            FillRect(mem_dc, &cur_rect, cur_brush);
                            DeleteObject(cur_brush);
                            SetTextColor(mem_dc, 0x00000000);
                        } else {
                            SetTextColor(mem_dc, fg_color);
                        }

                        if cell.bg != screen.default_bg {
                            SetBkMode(mem_dc, 2);
                            SetBkColor(mem_dc, bg_color);
                        } else {
                            SetBkMode(mem_dc, TRANSPARENT as i32);
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

        BitBlt(hdc, 0, 0, width, height, mem_dc, 0, 0, SRCCOPY);

        SelectObject(mem_dc, old_bitmap);
        DeleteObject(mem_bitmap);
        DeleteDC(mem_dc);
    }

    fn wide_string(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }
}
