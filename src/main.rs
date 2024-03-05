use glib::clone;
// glib and other dependencies are re-exported by the gtk crate
use gtk::glib;
use gtk::prelude::*;
use std::rc::Rc;

struct Context {
    name: String
}

fn control_widget(ctx: &Rc<Context>) -> gtk::Widget {
    let frame = gtk::Frame::new(Some("Controls"));

    return frame.into();
}

fn info_widget(ctx: &Rc<Context>) -> gtk::Widget {
    let frame = gtk::Frame::new(Some("Info"));

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
    println!("{}", ctx.name);
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
        name: String::from("ABC")
    });
    // Create a new application with the builder pattern
    let app = gtk::Application::builder()
        .application_id("com.github.gtk-rs.examples.basic")
        .build();
    app.connect_activate(clone!(@weak ctx => move |app| {
        on_activate(&app, &ctx);
    }));
    // Run the application
    app.run();
}
