use super::*;

#[test]
fn render() {
    let app = App::default();
    let mut buf = Buffer::empty(Rect::new(0, 0, 50, 4));

    app.render(buf.area, &mut buf);

    let mut expected = Buffer::with_lines(vec![
        "┏━━━━━━━━━━━━━ Counter App Tutorial ━━━━━━━━━━━━━┓",
        "┃                    Value: 0                    ┃",
        "┃                                                ┃",
        "┗━ Decrement <Left> Increment <Right> Quit <Q> ━━┛",
    ]);
    let title_style = Style::new().bold();
    let counter_style = Style::new().yellow();
    let key_style = Style::new().blue().bold();
    expected.set_style(Rect::new(14, 0, 22, 1), title_style);
    expected.set_style(Rect::new(28, 1, 1, 1), counter_style);
    expected.set_style(Rect::new(13, 3, 6, 1), key_style);
    expected.set_style(Rect::new(30, 3, 7, 1), key_style);
    expected.set_style(Rect::new(43, 3, 4, 1), key_style);

    // note ratatui also has an assert_buffer_eq! macro that can be used to
    // compare buffers and display the differences in a more readable way
    assert_eq!(buf, expected);
}

#[test]
fn handle_key_event() -> io::Result<()> {
    let mut app = App::default();

    app.handle_key_event(KeyCode::Right.into());
    assert_eq!(app.counter, 1);

    app.handle_key_event(KeyCode::Left.into());
    assert_eq!(app.counter, 0);

    let mut app = App::default();
    app.handle_key_event(KeyCode::Char('q').into());
    assert_eq!(app.exit, true);

    Ok(())
}
