use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
//     text::Span,
    Frame,
};

use crate::model::{Game, GameState};

pub fn draw(f: &mut Frame, game: &Game) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3), // Title
                Constraint::Min(0),    // Game Board
                Constraint::Length(3), // Instructions
            ]
            .as_ref(),
        )
        .split(f.area());

    // Title
    let title = Paragraph::new("RustDoku6")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);
    
    // Game Board Area
    draw_board(f, game, chunks[1]);

    // Instructions
    let status_text = match game.state {
        GameState::Playing => {
            let mode_str = match game.mode {
                crate::model::InputMode::Normal => "NORMAL",
                crate::model::InputMode::Pencil => "PENCIL",
            };
            format!("Mode: {} (p) | Mistakes: {} | Arrows/1-6/BS | q: Quit", mode_str, game.mistakes)
        },
        GameState::Won => format!("YOU WON! Mistakes: {} | Press 'q' to quit.", game.mistakes),
    };
    
    let instructions = Paragraph::new(status_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(if let GameState::Won = game.state { Color::Green } else { Color::White }))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(instructions, chunks[2]);
}

fn draw_board(f: &mut Frame, game: &Game, area: Rect) {
    // Inverted Grid Lines:
    // 1. Render a background color on the whole board area. This will show through the gaps.
    // 2. Use Layout with spacing to create gaps.
    // 3. Render cells as opaque blocks on top.

    // 1. Calculate Centered Board Area
    // We want a roughly square look. In terminals, chars are ~1:2 (W:H).
    // So for a square board, Width (chars) should be ~2x Height (rows).
    // Limit width to 60% of screen to prevent stretching.
    
    let (center_area, s) = calculate_board_rect(area, 60);

    // 2. Background (The "Lines")
    let grid_bg_color = Color::Blue;
    let bg_block = Block::default().style(Style::default().bg(grid_bg_color));
    f.render_widget(bg_block, center_area);
    
    // 3. Layouts with Spacing - Use Fixed Lengths for perfect uniformity
    let rows_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(s); 6])
        .spacing(1)
        .split(center_area);
        
    for r in 0..6 {
        let cols_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(2 * s); 6])
            .spacing(1) // Gap between cols
            .split(rows_layout[r]);
            
        for c in 0..6 {
            let cell = &game.grid.cells[r][c];
            // Determine content to render
            // If value is present, show it.
            // If value is None:
            //   If Pencil Mode: show marks.
            //   If Normal Mode: checks marks count. If 1, show it with validation color.
            
            // Region Coloring - More vibrant colors for better visibility
            let region_idx = (r / 2) * 2 + (c / 3);
            let region_bg = match region_idx {
                0 => Color::Rgb(30, 30, 80),   // Deeper blue
                1 => Color::Rgb(30, 80, 30),   // Richer green
                2 => Color::Rgb(80, 30, 30),   // Warmer red
                3 => Color::Rgb(80, 80, 30),   // Olive
                4 => Color::Rgb(30, 80, 80),   // Teal
                5 => Color::Rgb(80, 30, 80),   // Magenta
                _ => Color::Black,
            };

            // Cell Style Base
            let mut style = Style::default().bg(region_bg).add_modifier(Modifier::BOLD);
            
            // Cursor Highlight
            if (r, c) == game.cursor {
                style = style.bg(Color::Yellow).fg(Color::Black);
            } else if cell.is_fixed {
                style = style.fg(Color::Cyan); 
            } else {
                style = style.fg(Color::White);
            }
            
            // let cell_area = cols_layout[c]; (Moved down)

            // Content determination
            let mut rendered_text = String::new();
            let mut use_validation_style = false;
            let mut validation_valid = true;
            
            if let Some(v) = cell.value {
                rendered_text = v.to_string();
                
                // If it's a user-entered number (not fixed), check validity
                if !cell.is_fixed {
                    use_validation_style = true;
                    validation_valid = game.is_correct_move(r, c, v);
                }
            } else {
                // Check if exactly one mark is set (common logic for both modes now if we want validation)
                let _mark_count = cell.marks.iter().filter(|&&m| m).count();
                
                // Construct text based on mode, but we can reuse validation logic if count == 1
                match game.mode {
                    crate::model::InputMode::Pencil => {
                         for i in 0..6 {
                            if cell.marks[i] {
                                rendered_text.push_str(&format!("{}", i + 1));
                            } else {
                                rendered_text.push(' ');
                            }
                        }
                    }
                    crate::model::InputMode::Normal => {
                        // Check marks count. If 1, show it with validation color.
                        let mark_count = cell.marks.iter().filter(|&&m| m).count();
                        if mark_count == 1 {
                            let mark_idx = cell.marks.iter().position(|&m| m).unwrap();
                            let mark_val = (mark_idx + 1) as u8;
                            rendered_text = mark_val.to_string();
                            use_validation_style = true;
                            validation_valid = game.is_correct_move(r, c, mark_val);
                        }
                    }
                }
            }

            // Determine final background and foreground colors
            let _bg_color = region_bg;
            let _fg_color = Color::White;
            let _is_bold = true;
            
            // Determine final background and foreground colors
            let mut bg_color = region_bg;
            let mut fg_color = Color::White;
            let mut is_bold = true;
            
            if cell.is_fixed {
                fg_color = Color::Cyan;
            }

            // Validation Styling
            if use_validation_style {
                 if cell.value.is_some() {
                     // Explicit Value: Use Background Color
                     if validation_valid {
                        bg_color = Color::Green;
                        fg_color = Color::Black; 
                    } else {
                        bg_color = Color::Red;
                        fg_color = Color::White; 
                    }
                 } else {
                     // Implicit Value (Single Mark): Use Foreground Color only
                     // Keep the region background (or cursor background)
                     // But change text color to Green/Red
                     if validation_valid {
                         fg_color = Color::Green;
                     } else {
                         fg_color = Color::LightRed; // LightRed is brighter against dark backgrounds
                     }
                     // Maybe add Underline to indicate it's not final?
                     style = style.add_modifier(Modifier::UNDERLINED);
                 }
            } else if cell.value.is_none() && game.mode == crate::model::InputMode::Pencil {
                fg_color = Color::Gray;
                is_bold = false;
            }

            // Cursor Handling
            if (r, c) == game.cursor {
                bg_color = Color::Yellow;
                fg_color = Color::Black;
                
                // If validation is active, we need to ensure contrast or visibility on top of Yellow.
                if use_validation_style {
                     if cell.value.is_some() {
                        // Explicit: Background takes precedence over Cursor Yellow?
                        // Or Cursor Yellow takes precedence?
                        // If we want to show validation, we must modify Cursor color.
                        if validation_valid {
                             bg_color = Color::LightGreen; // Cursor on Valid
                        } else {
                             bg_color = Color::LightRed; // Cursor on Invalid
                        }
                     } else {
                        // Implicit: Foreground was Green/Red.
                        // On Yellow BG, Green text is hard to read. Red text is okay.
                        // Let's force Black/Dark Blue for contrast if it's Green?
                        // Or maybe use Blue for Valid on Yellow?
                        if validation_valid {
                            fg_color = Color::Rgb(0, 100, 0);
                        } else {
                            fg_color = Color::Red;
                        }
                     }
                }
            }
            
            let mut style = Style::default().bg(bg_color).fg(fg_color);
            if is_bold {
                style = style.add_modifier(Modifier::BOLD);
            }
            if cell.value.is_none() && game.mode == crate::model::InputMode::Pencil {
                 style = style.add_modifier(Modifier::ITALIC);
            }

            let cell_area = cols_layout[c];
            
            // Ensure full background coverage for the cell
            f.render_widget(Block::default().style(style), cell_area);
            
            // Render text
            if !rendered_text.trim().is_empty() {
                 let alignment = Alignment::Center;
                if cell_area.height > 1 {
                     let padding = (cell_area.height - 1) / 2;
                     let v_layout = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(padding),
                            Constraint::Length(1),
                            Constraint::Min(0),
                        ].as_ref())
                        .split(cell_area);
                     if v_layout.len() >= 2 {
                         f.render_widget(Paragraph::new(rendered_text).style(style).alignment(alignment), v_layout[1]);
                     }
                } else {
                     f.render_widget(Paragraph::new(rendered_text).style(style).alignment(alignment), cell_area);
                }
            }
        }
    }
}

// Helper to center a rect within another, maintaining aspect ratio logic
// width_percent: max % of width to use
// aspect_ratio: width / height (chars). For square in terminal, use 2.0.
fn centered_rect(r: Rect, width_percent: u16, aspect_ratio: f32) -> Rect {
    let available_width = r.width;
    let available_height = r.height;

    // 1. Determine Max Width
    let max_width = (available_width as f32 * (width_percent as f32 / 100.0)) as u16;
    
    // 2. Calculate Height based on aspect ratio (W = H * ratio  => H = W / ratio)
    let mut target_height = (max_width as f32 / aspect_ratio) as u16;
    let mut target_width = max_width;

    // 3. Constrain by available height
    if target_height > available_height {
        target_height = available_height;
        target_width = (target_height as f32 * aspect_ratio) as u16;
    }
    
    // 4. Center it
    let x = r.x + (r.width.saturating_sub(target_width)) / 2;
    let y = r.y + (r.height.saturating_sub(target_height)) / 2;
    
    Rect::new(x, y, target_width, target_height)
}

// Calculates a board size that guarantees perfectly uniform cells
// Formula: Total_Size = (6 * Cell_Size) + 5 gaps
// This ensures Integer Division by 6 has 0 remainder.
// Calculates a board size that guarantees perfectly uniform cells
// Formula: Total_Size = (6 * Cell_Size) + 5 gaps
// returns (BoardRect, scalar_s) where scalar_s is the height of a cell
fn calculate_board_rect(available: Rect, max_width_percent: u16) -> (Rect, u16) {
    let avail_w = (available.width as f32 * (max_width_percent as f32 / 100.0)) as u16;
    let avail_h = available.height;
    
    // Solve for 's' (scalar size)
    // Board_W = 6 * (2s) + 5 <= Available_W  =>  12s <= W - 5
    // Board_H = 6 * (1s) + 5 <= Available_H  =>  6s <= H - 5
    
    let s_w = if avail_w > 5 { (avail_w - 5) / 12 } else { 0 };
    let s_h = if avail_h > 5 { (avail_h - 5) / 6 } else { 0 };
    
    // Use the limiting scalar, minimum 1
    let s = std::cmp::max(1, std::cmp::min(s_w, s_h));
    
    let cell_h = s;
    let cell_w = 2 * s;
    
    let board_w = 6 * cell_w + 5;
    let board_h = 6 * cell_h + 5;
    
    let x = available.x + (available.width.saturating_sub(board_w)) / 2;
    let y = available.y + (available.height.saturating_sub(board_h)) / 2;
    
    (Rect::new(x, y, board_w, board_h), s)
}
