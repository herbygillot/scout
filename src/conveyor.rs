use log::debug;
use async_std::io;
use async_std::prelude::*;
use futures::channel::mpsc::Receiver;
use crate::config::Config;
use crate::common::{Result,Text};
use crate::events::Event;
use crate::state::State;
use crate::screen::Screen;
use crate::ui::Layout;

pub async fn task<W>(config: Config, outbound: W, mut wire: Receiver<Event>) -> Result<Option<Text>>
where
    W: io::Write + Send + Unpin + 'static,
{
    debug!("[task] start");

    let mut render: bool;
    let mut selection = None;

    let mut state = State::new();
    let mut screen = Screen::new(&config, outbound).await?;
    let mut layout = Layout::new(&config);

    layout.draw(&state)?;
    screen.render(&layout).await?;

    while let Some(event) = wire.next().await {
        render = false;

        match event {
            Event::Query(query) => {
                state.set_query(query);
                render = true;
            },
            Event::Matches(matches) => {
                state.set_matches(matches);
                render = true;
            },
            Event::Up => {
                state.select_up();
                render = true;
            },
            Event::Down => {
                state.select_down();
                render = true;
            },

            // NOTE: We don't need to break the loop since
            // the engine and input will drop the sender
            // and the loop will stop
            Event::Done => {
                selection = state.selection();
            },
            _ => (),
        };

        if render {
            layout.draw(&state)?;
            screen.render(&layout).await?;
        }
    };

    drop(wire);

    debug!("[task] end");

    Ok(selection)
}
