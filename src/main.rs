use glib::clone;
use gtk::gdk::Display;
use gtk::{glib, Align, Application, ApplicationWindow, Button, Grid, Label};
use gtk::{prelude::*, Box, CssProvider};
use serde::{Deserialize, Serialize};
use std::env::home_dir;
use std::fs;
use std::io::Error;
use std::io::Result;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};

#[derive(Serialize, Deserialize)]
struct ButtonMeta {
    label: String,
    id: Option<String>,
    col: i32,
    row: i32,
    cmd: String,
    args: Vec<String>,
    is_toggle: bool,
}

#[derive(Serialize, Deserialize)]
struct Config {
    margin_top: i32,
    shadow_width: i32,
    grid_width: i32,
    button_width: i32,
    button_height: i32,
    label_width: i32,
    label_height: i32,
    buttons: Vec<ButtonMeta>,
}

const APP_ID: &str = "org.gtk_rs.HelloWorld2";

pub fn execute_and_done(exe: &str, args: &[&str]) -> Error {
    Command::new(exe).args(args).exec()
}

pub fn execute(exe: &str, args: &[&str]) -> Result<ExitStatus> {
    Command::new(exe).args(args).spawn()?.wait()
}

fn read_file_string(filepath: &str) -> Result<String> {
    fs::read_to_string(filepath)
}

fn load_config() -> Result<Config> {
    let mut path: PathBuf = home_dir().unwrap();
    path.push(".config");
    path.push("gtk-hello");
    path.push("config.json");

    let data: String = read_file_string(path.to_str().unwrap()).unwrap();
    let p: Config = serde_json::from_str(&data)?;

    Ok(p)
}

fn get_config_style() -> Result<String> {
    let mut path: PathBuf = home_dir().unwrap();
    path.push(".config");
    path.push("gtk-hello");
    path.push("style.css");

    read_file_string(path.to_str().unwrap())
}

fn main() -> glib::ExitCode {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_startup(|_| load_css());
    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

fn load_css() {
    // Load the CSS file and add it to the
    let provider = CssProvider::new();
    provider.load_from_data(include_str!("style.css"));

    // Add the provider to the default screen
    gtk::style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    if let Ok(p) = get_config_style() {
        let custom_provider = CssProvider::new();
        custom_provider.load_from_data(&p);
        // Add the provider to the default screen
        gtk::style_context_add_provider_for_display(
            &Display::default().expect("Could not connect to a display."),
            &custom_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn build_ui(app: &Application) {
    let config_result: Result<Config> = load_config();
    let config: Config = config_result.unwrap();

    // Create a button with label and margins
    let invisible_button = Button::builder()
        .label(" ")
        .width_request(config.shadow_width)
        .css_name("closeButton")
        .build();

    let grid = Grid::builder()
        .width_request(config.grid_width)
        .margin_start(10)
        .margin_end(10)
        .margin_top(10)
        .margin_bottom(10)
        .vexpand(true)
        .valign(Align::End)
        .build();

    for button_meta in config.buttons {
        if button_meta.cmd != "" {
            let button = Button::builder()
                .label(button_meta.label)
                .width_request(config.button_width)
                .height_request(config.button_height)
                .margin_top(10)
                .margin_bottom(10)
                .margin_start(10)
                .margin_end(10)
                .build();

            grid.attach(&button, button_meta.col, button_meta.row, 1, 1);

            button.connect_clicked(move |_| {
                let args: Vec<&str> = button_meta.args.iter().map(|x| x.as_ref()).collect();
                if button_meta.is_toggle {
                    let _ = execute(&button_meta.cmd, &args);
                } else {
                    execute_and_done(&button_meta.cmd, &args);
                }
            });
        } else {
            let label = Label::builder()
                .label(button_meta.label)
                .xalign(0.0)
                .yalign(1.0)
                .width_request(config.label_width)
                .height_request(config.label_height)
                .margin_top(10)
                .margin_bottom(10)
                .margin_start(10)
                .margin_end(10)
                .css_name("label")
                .build();

            grid.attach(&label, button_meta.col, button_meta.row, 1, 1);
        }
    }

    let stage = Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .build();

    let container = Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .css_name("container")
        .build();

    let button = Button::builder()
        .label("<<")
        .width_request(config.button_width)
        .height_request(config.button_height / 2)
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(10)
        .margin_end(10)
        .css_name("expander")
        .build();

    grid.attach(&button, 0, -20, 1, 1);

    // container.append(&button);
    // container.append(&button2);
    container.append(&grid);
    stage.append(&invisible_button);
    stage.append(&container);

    // Create a window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("My GTK App")
        .child(&stage)
        .css_name("window")
        .build();

    button.connect_clicked(clone!(@weak invisible_button => move |button| {
        if invisible_button.width_request() != 100 {
            invisible_button.set_width_request(100);
            button.set_label(">>");
            grid.set_width_request(config.grid_width + config.shadow_width - 100);
        } else {
            invisible_button.set_width_request(config.shadow_width);
            button.set_label("<<");
            grid.set_width_request(config.grid_width);
        }
    }));

    // Connect to "clicked" signal of `button`
    invisible_button.connect_clicked(clone!(@weak window => move |_| {
        window.close();
    }));

    gtk4_layer_shell::init_for_window(&window);
    gtk4_layer_shell::set_layer(&window, gtk4_layer_shell::Layer::Overlay);
    gtk4_layer_shell::auto_exclusive_zone_enable(&window);
    gtk4_layer_shell::set_margin(&window, gtk4_layer_shell::Edge::Top, config.margin_top);

    let anchors = [
        (gtk4_layer_shell::Edge::Left, true),
        (gtk4_layer_shell::Edge::Right, true),
        (gtk4_layer_shell::Edge::Top, true),
        (gtk4_layer_shell::Edge::Bottom, true),
    ];

    for (anchor, state) in anchors {
        gtk4_layer_shell::set_anchor(&window, anchor, state);
    }

    // Present window
    window.present();
}
