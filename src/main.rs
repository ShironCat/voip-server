extern crate gstreamer as gst;
use gst::prelude::*;

use std::env;
use std::process;
use std::thread;

fn gst_init() {
    if let Err(e) = gst::init() {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn source(port: i32, user_id: u32) {
    gst_init();
    let pipeline = gst::Pipeline::new(Some(&format!("user-{}", user_id)));
    let source = gst::ElementFactory::make("udpsrc", None).unwrap();
    if let Ok(ip) = env::var("VOIP_IP") {
        source
            .set_property("address", &ip)
            .expect("Unable to set source address.");
    }
    source
        .set_property("port", &port)
        .expect("Unable to set source port.");
    let capsfilter = gst::ElementFactory::make("capsfilter", None).unwrap();
    let caps = gst::Caps::new_simple(
        "application/x-rtp",
        &[
            ("media", &String::from("audio")),
            ("payload", &(96 as i32)),
            ("clock-rate", &(48000 as i32)),
            (
                "encoding-name",
                &String::from("X-GST-OPUS-DRAFT-SPITTKA-00"),
            ),
        ],
    );
    capsfilter
        .set_property("caps", &caps)
        .expect("Unable to set capsfilter caps");
    let depayload = gst::ElementFactory::make("rtpopusdepay", None).unwrap();
    let decode = gst::ElementFactory::make("opusdec", None).unwrap();
    let sink = gst::ElementFactory::make("interaudiosink", None).unwrap();
    let channel = &format!("user-{}-stream", user_id);
    sink.set_property("channel", &channel)
        .expect("Unable to set interaudiosink channel.");
    pipeline
        .add_many(&[&source, &capsfilter, &depayload, &decode, &sink])
        .unwrap();
    source
        .link(&capsfilter)
        .expect("source could not be linked to capsfilter.");
    capsfilter
        .link(&depayload)
        .expect("capsfilter could not be linked to depay.");
    depayload
        .link(&decode)
        .expect("depay could not be linked to decode.");
    decode
        .link(&sink)
        .expect("decode could not be linked to sink.");
    pipeline
        .set_state(gst::State::Playing)
        .expect("Unable to play");
    let bus = pipeline.get_bus().unwrap();
    for msg in bus.iter_timed(gst::CLOCK_TIME_NONE) {
        use gst::MessageView;
        match msg.view() {
            MessageView::Eos(..) => break,
            MessageView::Error(err) => {
                eprintln!(
                    "Error from {:?}: {} ({:?})",
                    err.get_src().map(|s| s.get_path_string()),
                    err.get_error(),
                    err.get_debug()
                );
                break;
            }
            _ => (),
        }
    }
    pipeline
        .set_state(gst::State::Null)
        .expect("Unable to null.");
}

fn sink() {
    if let Err(e) = gst::init() {
        eprintln!("{}", e);
        process::exit(1);
    }
    let (major, minor, micro, nano) = gst::version();
    let mut nano_str = "";
    if nano == 1 {
        nano_str = "(CVS)";
    } else if nano == 2 {
        nano_str = "(Prerelease)";
    }
    println!(
        "This program is linked against GStreamer {}.{}.{} {}",
        major, minor, micro, nano_str
    );
    let pipeline = gst::Pipeline::new(Some("room-1"));
    let interleave = gst::ElementFactory::make("interleave", None).unwrap();
    interleave
        .set_property("name", &String::from("i"))
        .expect("Unable to set interleave name.");
    let sink = gst::ElementFactory::make("autoaudiosink", None).unwrap();
    let source_1 = gst::ElementFactory::make("interaudiosrc", None).unwrap();
    let channel_1 = "user-1-stream";
    source_1
        .set_property("channel", &channel_1)
        .expect("Unable to set interaudiosink channel.");
    let audioconvert_1 = gst::ElementFactory::make("audioconvert", None).unwrap();
    let capsfilter_1 = gst::ElementFactory::make("capsfilter", None).unwrap();
    let caps_1 = gst::Caps::new_simple(
        "audio/x-raw",
        &[("channels", &(1 as i32)), ("channel-mask", &0x1)],
    );
    capsfilter_1
        .set_property("caps", &caps_1)
        .expect("Unable to set capsfilter caps.");
    let queue_1 = gst::ElementFactory::make("queue", None).unwrap();
    let source_2 = gst::ElementFactory::make("interaudiosrc", None).unwrap();
    let channel_2 = "user-2-stream";
    source_2
        .set_property("channel", &channel_2)
        .expect("Unable to set interaudiosink channel.");
    let audioconvert_2 = gst::ElementFactory::make("audioconvert", None).unwrap();
    let capsfilter_2 = gst::ElementFactory::make("capsfilter", None).unwrap();
    let caps_2 = gst::Caps::new_simple(
        "audio/x-raw",
        &[("channels", &(1 as i32)), ("channel-mask", &0x2)],
    );
    capsfilter_2
        .set_property("caps", &caps_2)
        .expect("Unable to set capsfilter caps.");
    let queue_2 = gst::ElementFactory::make("queue", None).unwrap();
    pipeline
        .add_many(&[
            &interleave,
            &sink,
            &source_1,
            &audioconvert_1,
            &capsfilter_1,
            &queue_1,
            &source_2,
            &audioconvert_2,
            &capsfilter_2,
            &queue_2,
        ])
        .unwrap();
    interleave
        .link(&sink)
        .expect("interleave could not be linked to sink.");
    source_1
        .link(&audioconvert_1)
        .expect("source_1 could not be linked to audioconvert_1.");
    audioconvert_1
        .link(&capsfilter_1)
        .expect("audioconvert could not be linked to capsfilter_1.");
    capsfilter_1
        .link(&queue_1)
        .expect("capsfilter_1 could not be linked to queue_1.");
    queue_1
        .link(&interleave)
        .expect("queue_1 could not be linked to interleave.");
    source_2
        .link(&audioconvert_2)
        .expect("source_2 could not be linked to audioconvert_2.");
    audioconvert_2
        .link(&capsfilter_2)
        .expect("audioconvert could not be linked to capsfilter_1.");
    capsfilter_2
        .link(&queue_2)
        .expect("capsfilter_2 could not be linked to queue_2.");
    queue_2
        .link(&interleave)
        .expect("queue_2 could not be linked to interleave.");
    pipeline
        .set_state(gst::State::Playing)
        .expect("Unable to play");
    let bus = pipeline.get_bus().unwrap();
    for msg in bus.iter_timed(gst::CLOCK_TIME_NONE) {
        use gst::MessageView;
        match msg.view() {
            MessageView::Eos(..) => break,
            MessageView::Error(err) => {
                eprintln!(
                    "Error from {:?}: {} ({:?})",
                    err.get_src().map(|s| s.get_path_string()),
                    err.get_error(),
                    err.get_debug()
                );
                break;
            }
            _ => (),
        }
    }
    pipeline
        .set_state(gst::State::Null)
        .expect("Unable to null.");
}

fn main() {
    let handle_source_1 = thread::spawn(move || source(5100, 1));
    let handle_source_2 = thread::spawn(move || source(5101, 2));
    let handle_sink = thread::spawn(move || sink());
    handle_source_1.join().unwrap();
    handle_source_2.join().unwrap();
    handle_sink.join().unwrap();
}
