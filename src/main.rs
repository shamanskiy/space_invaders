use std::{error::Error, io, time::{Duration, Instant}, sync::mpsc, thread};

use crossterm::{terminal::{self, EnterAlternateScreen, LeaveAlternateScreen}, ExecutableCommand, cursor::{Hide, Show}, event::{self, Event, KeyCode}};
use rusty_audio::Audio;
use space_invaders::{frame::{self, new_frame, Drawable}, render::{self}, player::Player};

fn main() -> Result<(),Box<dyn Error>>{
    // audio setup
    let mut audio = Audio::new();
    audio.add("explode","sounds/explode.wav");
    audio.add("lose","sounds/lose.wav");
    audio.add("move","sounds/move.wav");
    audio.add("pew","sounds/pew.wav");
    audio.add("startup","sounds/startup.wav");
    audio.add("win","sounds/win.wav");
    audio.play("startup"); // plays in a separate thread

    // terminal setup
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;

    // Render in a separate thread. Uses simple, uncool, slow MPSC channels.
    let (render_tx, render_rx) = mpsc::channel();
    let render_handle = std::thread::spawn(move || {
        let mut last_frame = frame::new_frame();
        let mut stdout = io::stdout();
        render::render(&mut stdout, &last_frame, &last_frame, true);
        loop {
            let curr_frame = match render_rx.recv() {
                Ok(x) => x,
                Err(_) => break,
            };
            render::render(&mut stdout, &last_frame, &curr_frame, false);
            last_frame = curr_frame;
        }
    });

    let mut player = Player::new();
    let mut instant = Instant::now();
    'gameloop: loop {
        // per-frame setup
        let delta = instant.elapsed();
        instant = Instant::now();
        let mut curr_frame = new_frame();

        // input
        while event::poll(Duration::default())? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Left | KeyCode::Char('a') => player.move_left(),
                    KeyCode::Right | KeyCode::Char('d') => player.move_right(),
                    KeyCode::Char(' ') | KeyCode::Enter => {
                       if player.shoot() {
                        audio.play("pew");
                       }
                    },
                    KeyCode::Esc | KeyCode::Char('q') => {
                        audio.play("lose");
                        break 'gameloop;
                    }
                    _ => {}
                }
            }
        }

        // Updates
        player.update(delta);

        player.draw(&mut curr_frame);
        let _ = render_tx.send(curr_frame);
        // this limits number of FPS we generate to not overwhelm the render thread
        thread::sleep(Duration::from_millis(1)); 
    }

    // cleanup
    drop(render_tx); // closes the send channel, triggers render loop break and exits thread
    render_handle.join().unwrap();
    audio.wait();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
