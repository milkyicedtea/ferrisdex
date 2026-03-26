use std::error::Error;
use std::io::{self, Write};

use crossterm::{cursor, execute};
use image::DynamicImage;
use viuer::Config;

pub fn render_pokedex_view(
    img: &DynamicImage,
    info_lines: &[String],
) -> Result<(), Box<dyn Error>> {
    let img_width: u16 = 40;
    let img_height: u16 = 20;

    for _ in 0..img_height {
        println!();
    }

    let mut stdout = io::stdout();

    execute!(stdout, cursor::MoveUp(img_height))?;
    let (_, start_row) = cursor::position()?;

    let config = Config {
        width: Some(img_width as u32),
        use_kitty: true,
        absolute_offset: false,
        ..Default::default()
    };
    viuer::print(img, &config)?;

    let text_col = img_width + 2;
    for (i, line) in info_lines.iter().enumerate() {
        execute!(stdout, cursor::MoveTo(text_col, start_row + i as u16))?;
        print!("{}", line);
        stdout.flush()?;
    }

    let final_row = start_row + img_height.max(info_lines.len() as u16);
    execute!(stdout, cursor::MoveTo(0, final_row))?;
    println!();

    Ok(())
}
