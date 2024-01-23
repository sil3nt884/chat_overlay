use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use druid::{AppLauncher, WindowDesc, Widget, Color, Env, RenderContext, EventCtx, Event, LifeCycle, Selector, WidgetId, ExtEventSink};
use reqwest;
use std::time::{Duration, Instant};
use druid::piet::{Text, TextLayoutBuilder};
use serde::Deserialize;
use tokio::sync::mpsc;



#[derive(Debug, Clone, Deserialize)]
struct UserInfo {
    id: String,
    text: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ChatResponse {
    liveChatItems: Vec<HashMap<String, UserInfo>>,
    pageInfo: PageInfo,
    nextPageToken: String,
    hasPage: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct PageInfo {
    totalResults: u32,
    resultsPerPage: u32,
}


struct TransparentWindow {
    id: WidgetId,
    drag_state: DragState,
    chat_items: Arc<Mutex<Vec<(String, String)>>>
}
struct DragState {
    dragging: bool,
}



fn start_fetch_loop(chat_items: Arc<Mutex<Vec<(String, String)>>>) {
    tokio::spawn(async move {
        println!("spawned thread");
        println!("spawned thread received");
        loop {
            let url = "https://livenowbeta.asuscomm.com:3000/chat";
            match fetch_chat_items(url).await {
                Ok(chat_response) => {
                    println!("received chat items");
                    let mut chat_items_inner = Vec::new();

                    for item in chat_response.liveChatItems.iter() {
                        for (username, userinfo) in item {
                            chat_items_inner.push((username.clone(), userinfo.text.clone()))
                        }
                    }
                    *chat_items.lock().unwrap() = chat_items_inner;
                }
                Err(err) => {
                    println!("error in fetching chat items");
                    eprintln!("Error in fetching chat items: {}", err);
                }
            }
            tokio::time::sleep(Duration::from_secs(4)).await;
        }
    });
}


impl Widget<()> for TransparentWindow {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, _data: &mut (), _env: &Env) {
        match event {
            Event::WindowConnected => {
                start_fetch_loop(Arc::clone(&self.chat_items));
                ctx.request_timer(Duration::from_secs(1));
            }
            Event::MouseDown(mouse_event) => {
                if mouse_event.button.is_left() {
                    ctx.set_active(true);
                    self.drag_state.dragging = true;
                    ctx.request_update();
                    ctx.window().set_always_on_top(true);
                }
            }
            Event::MouseMove(mouse_event) => {
                if self.drag_state.dragging {
                    ctx.window().set_position(mouse_event.window_pos);
                    ctx.window().set_always_on_top(true);
                    ctx.request_update();
                }
            }

            Event::MouseUp(_) => {
                ctx.window().set_always_on_top(true);
                ctx.window().show_titlebar(false);
                self.drag_state.dragging = false;
                ctx.window().set_always_on_top(true);
            }

            Event::Timer(_) => {
                ctx.request_paint();
                ctx.request_timer(Duration::from_secs(1));
            }
            _ => {}
        }
    }

    fn lifecycle(&mut self, _ctx: &mut druid::LifeCycleCtx, _event: &druid::LifeCycle, _data: &(), _env: &Env) {
            match _event {
            LifeCycle::FocusChanged(gained, ..) => {
                _ctx.request_paint();
                if *gained {
                    _ctx.window().set_always_on_top(true);
                    _ctx.window().show_titlebar(true);
                } else {
                    _ctx.window().show_titlebar(false)
                }

                _ctx.request_paint();
            }
            _ => {}
        }
    }


    fn update(&mut self, _ctx: &mut druid::UpdateCtx, _old_data: &(), _data: &(), _env: &Env) {

    }

    fn layout(&mut self, _ctx: &mut druid::LayoutCtx, _bc: &druid::BoxConstraints, _data: &(), _env: &Env) -> druid::Size {
        druid::Size::new(300.0, 500.0) // Set your window size here
    }
    fn paint(&mut self, ctx: &mut druid::PaintCtx, _data: &(), _env: &Env) {
        let size = ctx.size();
        let mut y_position = druid::Point::new(30f64, 20f64);

        if let Ok(textlayout) = ctx
            .text()
            .new_text_layout("no chat users")
            .text_color(Color::WHITE)
            .build()
        {
            ctx.draw_text(&textlayout, y_position);

        }

        ctx.fill(size.to_rect(), &Color::rgba(0.0, 0.0, 0.0, 0.5));
        println!("drawing app {:?}", self.chat_items.lock().unwrap().iter().size_hint());

        let text_height = 24.0;
        let padding = 10.0;
        let max_messages_in_view = 500 / (text_height + padding) as usize;
        let chat_items = self.chat_items.lock().unwrap();

        let start = if chat_items.len() > max_messages_in_view {
            chat_items.len() - max_messages_in_view
        } else {
            0
        };

        for (username, text) in &chat_items[start..] {
            // Draw username
            println!("drawing username: {}", text);
            let mut user_data = String::new();
            user_data.push_str(username);
            user_data.push_str(": ");
            user_data.push_str(text);

            if let Ok(tl) = ctx
                .text()
                .new_text_layout(user_data)
                .text_color(Color::WHITE)
                .build()
            {
                ctx.draw_text(&tl, y_position);
                y_position.y += text_height + padding;
            }
        }

    }
}


    pub async fn fetch_chat_items(url: &str) -> Result<ChatResponse, reqwest::Error> {
        let response = reqwest::get(url).await?;
        let chat_response = response.json().await?;
        Ok(chat_response)
    }

    #[tokio::main]
    async fn main() {
        println!("starting program");
        let drag_state = DragState { dragging: false };
        let chat_items = Arc::new(Mutex::new(vec![]));

        let window_id = WidgetId::next(); // new code
        let transparent_window = TransparentWindow { id: window_id, drag_state, chat_items: Arc::clone(&chat_items) };
        print!("window created {:?}", window_id);
        let main_window = WindowDesc::new(transparent_window)
            .title("Transparent Window Example")
            .window_size(druid::Size::new(300.0, 500.0))
            .resizable(false)
            .show_titlebar(false)
            .transparent(true)
            .set_always_on_top(true);

        println!("window created");


        let app = AppLauncher::with_window(main_window)
            .log_to_console();

        app.launch(())
            .expect("Failed to launch application");


    }



