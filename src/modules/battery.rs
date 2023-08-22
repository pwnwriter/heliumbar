use json::{Error, JsonValue};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Seek};
use std::time::Duration;

use crate::builder::widgets_builder::{self, Align, WidgetConfig};
use crate::utils::file_handler::read_file_for_monitor;
use crate::utils::{command, listener, regex_matcher};
use glib::{MainContext, Receiver, Sender};
use gtk::prelude::*;
// use super::workspace::listen;

pub fn build_label(left: &gtk::Box, center: &gtk::Box, right: &gtk::Box, config: WidgetConfig) {
    let original: String = config.format.clone();

    let label = widgets_builder::build_and_align(&original, &left, &center, &right, &config);
    // println!("{}", text);
    update_widget(label, original, config.refresh_rate);
}

fn update_widget(label: gtk::Label, original: String, refresh_rate: i64) {
    let path = "/sys/class/power_supply/BAT0/capacity";

    let mut buffer = [0u8; 30];
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };
    let interval = if refresh_rate > 0 {
        std::time::Duration::from_secs(refresh_rate as u64)
    } else {
        std::time::Duration::from_secs(1)
    };
    let (sender, receiver) = MainContext::channel::<(String, String)>(glib::Priority::DEFAULT);
    //lister
    listener::listen(receiver, original, label); //listen and update according to it
    std::thread::spawn(move || {
        let mut previous_state: String = read_file_for_monitor(&mut file, &mut buffer);
        sender
            .send(("percentage".to_string(), previous_state.to_owned()))
            .unwrap_or_default();
        let mut current_state: String;
        loop {
            std::thread::sleep(interval);
            current_state = read_file_for_monitor(&mut file, &mut buffer);
            if previous_state == current_state {
                continue;
            }
            previous_state = current_state;
            sender
                .send(("percentage".to_string(), previous_state.to_owned()))
                .unwrap_or_default();
            // sender.send((previous_state.parse::<f64>().unwrap_or(1.0) / 1000000.0).to_string());
        } //loop
    }); //thread
} //update widget
