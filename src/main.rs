use glib::clone;
// glib and other dependencies are re-exported by the gtk crate
use gtk::glib;
use gtk::prelude::*;
use std::rc::Rc;
use gtk::gio;

struct Context<'a> {
    // controls
    method_selector: Option<&'a gtk::Widget>,
    // child process
    subprocess: Option<gio::Subprocess>,
    input: Option<gio::InputStream>,
    cancel_read: Option<gio::Cancellable>,
    line_input: Option<gio::DataInputStream>,
    header_processed: bool,
    suspend: bool
}

impl<'a> Default for Context<'a> {
    fn default() -> Context<'a> {
        Context {
            method_selector: None,
            //
            subprocess: None,
            input: None,
            cancel_read: None,
            line_input: None,
            header_processed: false,
            suspend: false
        }
    }
}

fn control_widget(ctx: &Rc<Context>) -> gtk::Widget {
    let frame = gtk::Frame::new(Some("Controls"));
    let bx = gtk::Box::new(gtk::Orientation::Vertical, 0);

    frame.set_child(Some(&bx));
    bx.append(&gtk::Label::new(Some("Preset:")));
    let presets = ["2 Bodies", "3 Bodies", "Solar", "Saturn"];
    let preset_selector = gtk::DropDown::from_strings(&presets);

    // TODO: signal
    bx.append(&preset_selector);

    let methods = ["Euler", "Verlet"];
    bx.append(&gtk::Label::new(Some("Method:")));
    // TODO: store in ctx
    let method_selector = gtk::DropDown::from_strings(&methods);
//    ctx.method_selector = Some(clone!(@weak (&method_selector.into())));
    bx.append(&method_selector);
    bx.append(&gtk::Label::new(Some("Input:")));
    let entry = gtk::Entry::new();
    // TODO signal
    // TODO store buffer
    // let buffer = entry.get_buffer();
    // buffer.set_text() // TODO
    bx.append(&entry);
    bx.append(&gtk::Label::new(Some("dt:")));
    // TODO store dt
    let dt = gtk::SpinButton::with_range(1e-14, 0.1, 0.00001);
    dt.set_digits(8);
    // dt.set_value(); // TODO
    // signal
    bx.append(&dt);

    return frame.into();
}

fn info_widget(ctx: &Rc<Context>) -> gtk::Widget {
    let frame = gtk::Frame::new(Some("Info"));
    let bx = gtk::Box::new(gtk::Orientation::Vertical, 0);
    frame.set_child(Some(&bx));

    let strings = [];
    // TODO: store body_selector
    let body_selector = gtk::DropDown::from_strings(&strings);
    body_selector.set_selected(0);
    // signal

    bx.append(&body_selector);

    for i in 0..3 {
        let x = gtk::Label::new(Some("-"));
        bx.append(&x);
        // store x
        x.set_width_chars(30);
        x.set_use_markup(true);
    }
    for i in 0..3 {
        let vx = gtk::Label::new(Some("-"));
        bx.append(&vx);
        // store vx
        vx.set_width_chars(30);
        vx.set_use_markup(true);
    }

    return frame.into();
}

fn right_pane(ctx: &Rc<Context>) -> gtk::Widget {
    let bx = gtk::Box::new(gtk::Orientation::Vertical, 0);

    bx.append(&control_widget(ctx));
    bx.append(&info_widget(ctx));

    bx.set_halign(gtk::Align::End);
    bx.set_valign(gtk::Align::Start);

    return bx.into();
}

// When the application is launched…
fn on_activate(application: &gtk::Application, ctx: &Rc<Context>) {
    let window = gtk::ApplicationWindow::new(application);
    window.set_title(Some("N-Body"));
    window.set_default_size(1024, 768);

    let drawing_area = gtk::DrawingArea::new();
    drawing_area.set_vexpand(true);
    drawing_area.set_hexpand(true);

    let overlay = gtk::Overlay::new();

    window.set_child(Some(&overlay));

    overlay.set_child(Some(&drawing_area));
    overlay.add_overlay(&right_pane(ctx));

    // set_draw_func

    let motion = gtk::EventControllerMotion::new();

    let glick = gtk::GestureClick::new();

    let scroll = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);

    // drawing_area.add_controller(Some(&scroll));

    let zoom = gtk::GestureZoom::new();

    // … with a button in it …
    //let button = gtk::Button::with_label("Hello World!");
    // … which closes the window when clicked
    //button.connect_clicked(clone!(@weak window => move |_| window.close()));
    //window.set_child(Some(&button));
    window.present();
}

fn main() {
    let ctx = Rc::new(Context{
        ..Default::default()
    });

    gtk::disable_setlocale();

    let app = gtk::Application::builder()
        .application_id("n-bodies")
        .build();
    app.connect_activate(clone!(@weak ctx => move |app| {
        on_activate(&app, &ctx);
    }));
    // Run the application
    app.run();
}
