use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::video::{Window, WindowContext};
use sdl2::image::{LoadTexture, InitFlag};
use sdl2::mixer::{InitFlag as MixerFlag, AUDIO_S16LSB, DEFAULT_CHANNELS};
use sdl2::render::Texture;
use sdl2::ttf;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use std::collections::HashMap;
use gilrs::{Gilrs, Button, Event as GilrsEvent, EventType};

const SCREEN_WIDTH: u32 = 981;
const SCREEN_HEIGHT: u32 = 673;
const BOX_SIZE: (u32, u32) = (267, 400);
const SHAD_SIZE: (u32, u32) = (294, 440);
const HOVER_BOX_SIZE: (u32, u32) = (294, 440);
const TRANSITION_SPEED: f32 = 0.15; // Higher = faster transition

#[derive(Debug, Serialize, Deserialize, Clone)]
struct LauncherOptions {
    selector: u8,
    bgtype: u8,
    background_color: (u8, u8, u8),
    onload: u8,
}

impl Default for LauncherOptions {
    fn default() -> Self {
        LauncherOptions {
            selector: 1,
            bgtype: 1,
            background_color: (66, 113, 183),
            onload: 1,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct GameOptions {
    autosave: bool,
    disable_frame_delay: bool,
    save_playthrough: bool,
    window_size: String,
    fullscreen: u8,
    window_scale: u8,
    new_renderer: bool,
    ignore_aspect_ratio: bool,
    no_sprite_limits: bool,
    output_method: String,
    linear_filtering: bool,
    shader: String,
    enable_audio: bool,
    audio_freq: u32,
    audio_channels: u8,
    audio_samples: u32,
    controls: String,
    gamepad_controls: String,
}

impl Default for GameOptions {
    fn default() -> Self {
        GameOptions {
            autosave: true,
            disable_frame_delay: false,
            save_playthrough: false,
            window_size: "1024x960".to_string(),
            fullscreen: 0,
            window_scale: 3,
            new_renderer: true,
            ignore_aspect_ratio: false,
            no_sprite_limits: false,
            output_method: "SDL".to_string(),
            linear_filtering: true,
            shader: "None".to_string(),
            enable_audio: true,
            audio_freq: 44100,
            audio_channels: 2,
            audio_samples: 2048,
            controls: String::new(),
            gamepad_controls: String::new(),
        }
    }
}

struct Launcher {
    install_dir: PathBuf,
    sfc_dir: PathBuf,
    launcher_dir: PathBuf,
    launcher_options: LauncherOptions,
    gamepad_system: Option<Gilrs>,
    selected_game: usize,
    mouse_x: i32,
    mouse_y: i32,
    color_transitions: HashMap<usize, f32>, // Track color blend for each game (0.0 = grayscale, 1.0 = full color)
}

impl Launcher {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let install_dir = Self::get_install_dir()?;
        let sfc_dir = install_dir.join("sfcs");
        let launcher_dir = install_dir.join("launcher");
        
        fs::create_dir_all(&sfc_dir)?;
        fs::create_dir_all(&launcher_dir)?;
        fs::create_dir_all(&launcher_dir.join("UI"))?;
        fs::create_dir_all(&launcher_dir.join("pngs"))?;
        let launcher_options = Self::load_launcher_options(&launcher_dir)?;
        
        let gamepad_system = Gilrs::new().ok();
        if gamepad_system.is_none() {
            eprintln!("Warning: Could not initialize gamepad support");
        } else {
            println!("Gamepad system initialized successfully");
        }
        
        Ok(Launcher {
            install_dir,
            sfc_dir,
            launcher_dir,
            launcher_options,
            gamepad_system,
            selected_game: 0,
            mouse_x: 0,
            mouse_y: 0,
            color_transitions: HashMap::new(),
        })
    }
    
    fn get_install_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let current_dir = std::env::current_dir()?;
        Ok(current_dir)
    }
    
    fn load_launcher_options(launcher_dir: &Path) -> Result<LauncherOptions, Box<dyn std::error::Error>> {
        let options_path = launcher_dir.join("launcher.json");
        
        if options_path.exists() {
            let content = fs::read_to_string(options_path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(LauncherOptions::default())
        }
    }
    
    fn save_launcher_options(&self) -> Result<(), Box<dyn std::error::Error>> {
        let options_path = self.launcher_dir.join("launcher.json");
        let content = serde_json::to_string_pretty(&self.launcher_options)?;
        fs::write(options_path, content)?;
        Ok(())
    }
    
    fn load_game_options(install_dir: &Path) -> Result<GameOptions, Box<dyn std::error::Error>> {
        let ini_path = install_dir.join("smw.ini");
        
        if ini_path.exists() {
            Ok(GameOptions::default())
        } else {
            Ok(GameOptions::default())
        }
    }
    
    fn scan_sfc_files(&self) -> Vec<String> {
        let mut sfcs = Vec::new();
        
        if let Ok(entries) = fs::read_dir(&self.sfc_dir) {
            for entry in entries.flatten() {
                if let Some(file_name) = entry.file_name().to_str() {
                    if file_name.to_lowercase().ends_with(".sfc") {
                        sfcs.push(file_name.to_string());
                    }
                }
            }
        }
        
        let priority = ["smb1.sfc", "smbll.sfc", "smw.sfc"];
        sfcs.sort_by(|a, b| {
            let a_idx = priority.iter().position(|&x| x == a).unwrap_or(priority.len());
            let b_idx = priority.iter().position(|&x| x == b).unwrap_or(priority.len());
            a_idx.cmp(&b_idx)
        });
        
        sfcs
    }
    
    fn launch_game(&self, sfc_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let sfc_path = self.sfc_dir.join(sfc_name);
        let exe_name = if cfg!(target_os = "windows") { "smw.exe" } else { "smw" };
        let smw_path = self.install_dir.join(exe_name);
        
        if !smw_path.exists() {
            eprintln!("SMW executable not found at: {}", smw_path.display());
            return Err("SMW executable not found".into());
        }
        
        println!("Launching: {} with ROM: {}", exe_name, sfc_name);
        
        Command::new(smw_path)
            .arg(sfc_path)
            .current_dir(&self.install_dir)
            .spawn()?;
        
        Ok(())
    }
    
    fn handle_gamepad_input(&mut self) -> Option<GamepadAction> {
        if let Some(ref mut gilrs) = self.gamepad_system {
            while let Some(GilrsEvent { event, .. }) = gilrs.next_event() {
                match event {
                    EventType::ButtonPressed(button, _) => {
                        return Some(match button {
                            Button::South => GamepadAction::Confirm,
                            Button::East => GamepadAction::Back,
                            Button::DPadUp | Button::North => GamepadAction::Up,
                            Button::DPadDown => GamepadAction::Down,
                            Button::DPadLeft | Button::West => GamepadAction::Left,
                            Button::DPadRight => GamepadAction::Right,
                            Button::Start => GamepadAction::Start,
                            _ => GamepadAction::None,
                        });
                    }
                    _ => {}
                }
            }
        }
        None
    }

    fn get_game_box_rect(&self, idx: usize) -> Option<Rect> {
        if idx >= 3 {
            return None;
        }
        
        let col = (idx % 3) + 1;
        let box_x = match col {
            1 => 30,
            2 => 357,
            _ => 684,
        } as i32;
        let box_y = 143;
        
        Some(Rect::new(box_x, box_y, BOX_SIZE.0, BOX_SIZE.1))
    }

    fn update_selection_from_mouse(&mut self, sfcs: &[String]) {
        for (idx, _) in sfcs.iter().enumerate().take(3) {
            if let Some(rect) = self.get_game_box_rect(idx) {
                if rect.contains_point((self.mouse_x, self.mouse_y)) {
                    self.selected_game = idx;
                    break;
                }
            }
        }
    }
    
    fn update_color_transitions(&mut self, num_games: usize) {
        for idx in 0..num_games {
            let target = if idx == self.selected_game { 1.0 } else { 0.0 };
            let current = self.color_transitions.entry(idx).or_insert(0.0);
            
            // Smooth lerp towards target
            if (*current - target).abs() > 0.01 {
                *current += (target - *current) * TRANSITION_SPEED;
            } else {
                *current = target;
            }
        }
    }
    
    fn get_color_blend(&self, idx: usize) -> f32 {
        *self.color_transitions.get(&idx).unwrap_or(&0.0)
    }
}

#[derive(Debug)]
enum GamepadAction {
    Confirm,
    Back,
    Up,
    Down,
    Left,
    Right,
    Start,
    None,
}

struct UIButton {
    rect: Rect,
    label: String,
    normal_color: Color,
    hover_color: Color,
    pressed_color: Color,
}

impl UIButton {
    fn new(x: i32, y: i32, width: u32, height: u32, label: &str) -> Self {
        UIButton {
            rect: Rect::new(x, y, width, height),
            label: label.to_string(),
            normal_color: Color::RGB(100, 100, 150),
            hover_color: Color::RGB(150, 150, 200),
            pressed_color: Color::RGB(200, 200, 250),
        }
    }
    
    fn is_hovered(&self, mouse_x: i32, mouse_y: i32) -> bool {
        self.rect.contains_point((mouse_x, mouse_y))
    }
    
    fn draw(&self, canvas: &mut Canvas<Window>, mouse_x: i32, mouse_y: i32, pressed: bool) {
        let color = if pressed && self.is_hovered(mouse_x, mouse_y) {
            self.pressed_color
        } else if self.is_hovered(mouse_x, mouse_y) {
            self.hover_color
        } else {
            self.normal_color
        };
        
        canvas.set_draw_color(color);
        canvas.fill_rect(self.rect).unwrap();
        
        canvas.set_draw_color(Color::RGB(50, 50, 50));
        canvas.draw_rect(self.rect).unwrap();
    }

    fn draw_with_text<'a>(
        &self,
        canvas: &mut Canvas<Window>,
        font: &ttf::Font,
        mouse_x: i32,
        mouse_y: i32,
        pressed: bool,
        texture_creator: &'a TextureCreator<WindowContext>,
    ) -> Result<(), String> {
        self.draw(canvas, mouse_x, mouse_y, pressed);
        
        let surface = font
            .render(&self.label)
            .blended(Color::RGB(255, 255, 255))
            .map_err(|e| e.to_string())?;
        
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())?;
        
        let text_query = texture.query();
        let text_rect = Rect::new(
            self.rect.x() + (self.rect.width() as i32 - text_query.width as i32) / 2,
            self.rect.y() + (self.rect.height() as i32 - text_query.height as i32) / 2,
            text_query.width,
            text_query.height,
        );
        
        canvas.copy(&texture, None, Some(text_rect))?;
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SMAS Launcher (Rust) - Grayscale Selection ===");
    println!("Initializing...");
    
    let mut launcher = Launcher::new()?;
    
    println!("Install directory: {}", launcher.install_dir.display());
    println!("SFC directory: {}", launcher.sfc_dir.display());
    println!("Launcher directory: {}", launcher.launcher_dir.display());
    
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = sdl2::image::init(InitFlag::PNG)?;
    let ttf_context = ttf::init().map_err(|e| e.to_string())?;
    
    let frequency = 44_100;
    let format = AUDIO_S16LSB;
    let channels = DEFAULT_CHANNELS;
    let chunk_size = 1_024;
    
    sdl2::mixer::open_audio(frequency, format, channels, chunk_size)?;
    let _mixer_context = sdl2::mixer::init(MixerFlag::MP3 | MixerFlag::OGG)?;
    sdl2::mixer::allocate_channels(4);
    
    // Load background music
    let music_path = launcher.launcher_dir.join("smas.wav");
    let music = if music_path.exists() {
        match sdl2::mixer::Music::from_file(&music_path) {
            Ok(m) => {
                println!("Loaded background music: {}", music_path.display());
                Some(m)
            }
            Err(e) => {
                eprintln!("Failed to load background music: {}", e);
                None
            }
        }
    } else {
        eprintln!("Background music not found at: {}", music_path.display());
        None
    };
    
    // Load launch sound effect
    let launch_sound_path = launcher.launcher_dir.join("pg.wav");
    let launch_sound = if launch_sound_path.exists() {
        match sdl2::mixer::Chunk::from_file(&launch_sound_path) {
            Ok(s) => {
                println!("Loaded launch sound: {}", launch_sound_path.display());
                Some(s)
            }
            Err(e) => {
                eprintln!("Failed to load launch sound: {}", e);
                None
            }
        }
    } else {
        eprintln!("Launch sound not found at: {}", launch_sound_path.display());
        None
    };
    
    // Play music if loaded
    if let Some(ref m) = music {
        m.play(-1)?; // -1 for infinite loop
    }
    
    let display_mode = video_subsystem.current_display_mode(0)?;
    let refresh_rate = display_mode.refresh_rate;
    println!("Display refresh rate: {}Hz", refresh_rate);
    
    let target_frame_time = Duration::from_secs_f64(1.0 / refresh_rate as f64);
    
    let window = video_subsystem
        .window("SMAS Launcher", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .build()?;
    
    let mut canvas = window.into_canvas()
        .accelerated()
        .present_vsync()
        .build()?;
    
    let texture_creator = canvas.texture_creator();
    
    sdl_context.mouse().show_cursor(false);
    
    let cursor_path = launcher.launcher_dir.join("UI").join("Cursor.png");
    let cursor_texture = if cursor_path.exists() {
        match texture_creator.load_texture(&cursor_path) {
            Ok(t) => {
                println!("Loaded cursor texture: {}", cursor_path.display());
                Some(t)
            }
            Err(e) => {
                eprintln!("Failed to load cursor texture: {}", e);
                None
            }
        }
    } else {
        eprintln!("Cursor texture not found at: {}", cursor_path.display());
        None
    };
    
    let bg_path = launcher.launcher_dir.join("MBG.png");
    let bg_texture = if bg_path.exists() && launcher.launcher_options.bgtype == 2 {
        match texture_creator.load_texture(&bg_path) {
            Ok(t) => {
                println!("Loaded background texture: {}", bg_path.display());
                Some(t)
            }
            Err(e) => {
                eprintln!("Failed to load background texture: {}", e);
                None
            }
        }
    } else {
        None
    };
    
    let pointer_path = launcher.launcher_dir.join("pointer.png");
    let pointer_texture = if pointer_path.exists() && launcher.launcher_options.selector == 1 {
        match texture_creator.load_texture(&pointer_path) {
            Ok(t) => {
                println!("Loaded pointer texture: {}", pointer_path.display());
                Some(t)
            }
            Err(e) => {
                eprintln!("Failed to load pointer texture: {}", e);
                None
            }
        }
    } else {
        None
    };
    
    let font_path = launcher.launcher_dir.join("smw.ttf");
    let font = if font_path.exists() {
        match ttf_context.load_font(&font_path, 24) {
            Ok(f) => {
                println!("Loaded font: {}", font_path.display());
                Some(f)
            }
            Err(e) => {
                eprintln!("Failed to load font: {}", e);
                None
            }
        }
    } else {
        eprintln!("Font not found at: {}", font_path.display());
        None
    };
    
    let mut event_pump = sdl_context.event_pump()?;
    let mouse_pressed = false;
    
    let sfcs = launcher.scan_sfc_files();
    
    if sfcs.is_empty() {
        println!("\nWARNING: No SFC files found!");
        println!("Please add .sfc ROM files to: {}", launcher.sfc_dir.display());
    } else {
        println!("\nFound {} game(s):", sfcs.len());
        for (idx, sfc) in sfcs.iter().enumerate() {
            println!("  {}. {}", idx + 1, sfc);
        }
    }
    
    let mut covers: HashMap<String, Texture> = HashMap::new();
    for sfc in &sfcs {
        let name = sfc.trim_end_matches(".sfc");
        let path = launcher.launcher_dir.join("pngs").join(format!("{}.png", name));
        if path.exists() {
            if let Ok(mut tex) = texture_creator.load_texture(&path) {
                tex.set_blend_mode(sdl2::render::BlendMode::Blend);
                covers.insert(sfc.clone(), tex);
            }
        }
    }
    
    let options_btn = UIButton::new(
        (SCREEN_WIDTH / 2 - 75) as i32,
        593,
        150,
        40,
        "Options"
    );
    
    let launcher_opts_btn = UIButton::new(
        (SCREEN_WIDTH / 4 * 3 - 75) as i32,
        593,
        150,
        40,
        "Launcher"
    );
    
    let update_btn = UIButton::new(
        (SCREEN_WIDTH / 4 - 65) as i32,
        593,
        130,
        40,
        "Update"
    );
    
    println!("\nLauncher ready with grayscale selection!");
    println!("Controls:");
    println!("  - Click game box to launch");
    println!("  - Arrow keys or gamepad D-Pad to navigate");
    println!("  - Enter or gamepad A/X to launch");
    println!("  - ESC or gamepad B/Circle to quit");
    
    let mut should_launch: Option<usize> = None;
    
    'running: loop {
        let frame_start = std::time::Instant::now();
        
        // Update color transitions for smooth animation
        launcher.update_color_transitions(sfcs.len());
        
        if let Some(action) = launcher.handle_gamepad_input() {
            match action {
                GamepadAction::Confirm => {
                    if !sfcs.is_empty() {
                        should_launch = Some(launcher.selected_game);
                    }
                }
                GamepadAction::Left => {
                    if launcher.selected_game > 0 {
                        launcher.selected_game -= 1;
                        println!("Selected: {}", sfcs[launcher.selected_game]);
                    }
                }
                GamepadAction::Right => {
                    if launcher.selected_game < sfcs.len().saturating_sub(1) {
                        launcher.selected_game += 1;
                        println!("Selected: {}", sfcs[launcher.selected_game]);
                    }
                }
                GamepadAction::Back => break 'running,
                _ => {}
            }
        }
        
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::MouseButtonDown { mouse_btn: sdl2::mouse::MouseButton::Left, x, y, .. } => {
                    // Check if clicked on a game box
                    for (idx, sfc) in sfcs.iter().enumerate().take(3) {
                        if let Some(rect) = launcher.get_game_box_rect(idx) {
                            if rect.contains_point((x, y)) {
                                launcher.selected_game = idx;
                                should_launch = Some(idx);
                                break;
                            }
                        }
                    }
                }
                Event::MouseMotion { x, y, .. } => {
                    launcher.mouse_x = x;
                    launcher.mouse_y = y;
                    launcher.update_selection_from_mouse(&sfcs);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    if launcher.selected_game > 0 {
                        launcher.selected_game -= 1;
                        println!("Selected: {}", sfcs[launcher.selected_game]);
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    if launcher.selected_game < sfcs.len().saturating_sub(1) {
                        launcher.selected_game += 1;
                        println!("Selected: {}", sfcs[launcher.selected_game]);
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Return),
                    ..
                } => {
                    if !sfcs.is_empty() {
                        should_launch = Some(launcher.selected_game);
                    }
                }
                _ => {}
            }
        }
        
        let mouse_state = event_pump.mouse_state();
        let (mouse_x, mouse_y) = (mouse_state.x(), mouse_state.y());
        
        launcher.mouse_x = mouse_x;
        launcher.mouse_y = mouse_y;
        
        canvas.set_draw_color(Color::RGB(
            launcher.launcher_options.background_color.0,
            launcher.launcher_options.background_color.1,
            launcher.launcher_options.background_color.2,
        ));
        canvas.clear();

        for (idx, sfc) in sfcs.iter().enumerate().take(3) {
            let col = idx;
            let x = match col { 0 => 30, 1 => 357, _ => 684 };
            let y = 143;

            let rect = Rect::new(x, y, BOX_SIZE.0, BOX_SIZE.1);
            let is_selected = idx == launcher.selected_game;
            let color_blend = launcher.get_color_blend(idx);

            canvas.set_draw_color(Color::RGB(200, 200, 200));
            canvas.fill_rect(rect)?;
            canvas.set_draw_color(Color::RGB(100, 100, 100));
            canvas.draw_rect(rect)?;

            if let Some(tex) = covers.get_mut(sfc) {
                let dst = Rect::new(
                    x + 10,
                    y + 10,
                    BOX_SIZE.0 - 20,
                    BOX_SIZE.1 - 70,
                );
                
                // Apply grayscale effect to unselected ROMs
                // color_blend: 0.0 = grayscale, 1.0 = full color
                // When selected, color_blend = 1.0 (full color)
                // When unselected, color_blend = 0.0 (grayscale)
                
                // Simple grayscale: average of RGB creates gray tone
                // We use equal RGB values for true grayscale
                let gray_intensity = 128; // Brightness for grayscale (0-255)
                
                // Interpolate between gray and full color
                let r_mod = (gray_intensity as f32 + (255.0 - gray_intensity as f32) * color_blend) as u8;
                let g_mod = (gray_intensity as f32 + (255.0 - gray_intensity as f32) * color_blend) as u8;
                let b_mod = (gray_intensity as f32 + (255.0 - gray_intensity as f32) * color_blend) as u8;
                
                tex.set_color_mod(r_mod, g_mod, b_mod);
                tex.set_alpha_mod(255);
                
                canvas.copy(tex, None, dst)?;
                
                // Reset color mod for next frame
                tex.set_color_mod(255, 255, 255);
            }

            if let Some(f) = &font {
                let surf = f.render(sfc.trim_end_matches(".sfc"))
                    .blended(Color::RGB(0, 0, 0))?;
                let tex = texture_creator.create_texture_from_surface(&surf)?;
                let q = tex.query();
                let tr = Rect::new(
                    x + (BOX_SIZE.0 as i32 - q.width as i32) / 2,
                    y + BOX_SIZE.1 as i32 - 50,
                    q.width,
                    q.height,
                );
                canvas.copy(&tex, None, tr)?;
            }

            if is_selected {
                canvas.set_draw_color(Color::RGB(255, 220, 0));
                let thickness = 3;
                for i in 0..thickness {
                    let thick_rect = Rect::new(
                        rect.x() - i,
                        rect.y() - i,
                        rect.width() + (i * 2) as u32,
                        rect.height() + (i * 2) as u32
                    );
                    canvas.draw_rect(thick_rect)?;
                }
            }
        }

        canvas.present();
        
        // Handle launching after rendering
        if let Some(game_idx) = should_launch.take() {
            // Fade out music and play launch sound
            sdl2::mixer::Music::fade_out(500)?; // 500ms fade out
            if let Some(ref sound) = launch_sound {
                sdl2::mixer::Channel::all().play(&sound, 0)?;
            }
            
            // Small delay to let sound play
            std::thread::sleep(Duration::from_millis(100));
            
            if let Err(e) = launcher.launch_game(&sfcs[game_idx]) {
                eprintln!("Failed to launch game: {}", e);
            } else if launcher.launcher_options.onload == 1 {
                break 'running;
            }
        }
        
        std::thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}