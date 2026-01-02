mod model;
mod ui;

use std::{error::Error, io, time::Duration};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

use crate::model::Game;

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create App
    let mut game = Game::new();

    // Run Loop
    let res = run_app(&mut terminal, &mut game);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, game: &mut Game) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, game)).map_err(|e| io::Error::other(e.to_string()))?;

        if event::poll(Duration::from_millis(250))?
            && let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('p') => game.toggle_mode(),
                        KeyCode::Left => game.move_cursor(0, -1),
                        KeyCode::Right => game.move_cursor(0, 1),
                        KeyCode::Up => game.move_cursor(-1, 0),
                        KeyCode::Down => game.move_cursor(1, 0),
                        KeyCode::Char('1') => game.handle_input(1),
                        KeyCode::Char('2') => game.handle_input(2),
                        KeyCode::Char('3') => game.handle_input(3),
                        KeyCode::Char('4') => game.handle_input(4),
                        KeyCode::Char('5') => game.handle_input(5),
                        KeyCode::Char('6') => game.handle_input(6),
                        KeyCode::Backspace | KeyCode::Delete => game.clear_cell(),
                        _ => {}
                    }
                }
    }
}
