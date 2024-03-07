use glib::clone;
// glib and other dependencies are re-exported by the gtk crate
use gtk::glib;
use gtk::glib::SourceId;
use gtk::prelude::*;
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::ffi::OsStr;
use gtk::gio;

pub struct SharedFromThisBase<T> {
    weak: RefCell<Weak<T>>,
}

impl<T> SharedFromThisBase<T> {
    pub fn new() -> SharedFromThisBase<T> {
        SharedFromThisBase {
            weak: RefCell::new(Weak::new()),
        }
    }

    pub fn initialise(&self, r: &Rc<T>) {
        *self.weak.borrow_mut() = Rc::downgrade(r);
    }
}

pub trait SharedFromThis<T> {
    fn get_base(&self) -> &SharedFromThisBase<T>;

    fn shared_from_this(&self) -> Rc<T> {
        self.get_base().weak.borrow().upgrade().unwrap()
    }
}

struct Body {
    x0: f64,
    y0: f64,
    show_tip: bool,
    name: String,
    r: [f64;3],
    v: [f64;3],
    m: f64,
    // color
    cr: f64,
    cg: f64,
    cb: f64,
    // radius
    rad: f64
}

impl Body {
    fn new() -> Body {
        Body {
            x0: 0.0,
            y0: 0.0,
            show_tip: false,
            name: String::new(),
            r: [0.0, 0.0, 0.0],
            v: [0.0, 0.0, 0.0],
            m: 0.0,
            cr: 0.0,
            cg: 0.0,
            cb: 0.0,
            rad: 0.0
        }
    }
}

impl SharedFromThis<RefCell<Context>> for Context {
    fn get_base(&self) -> &SharedFromThisBase<RefCell<Context>> {
        &self.base
    }
}

struct Context {
    base: SharedFromThisBase<RefCell<Context>>,
    bodies: Vec<Body>,
    method: u32,
    active_body: i32,
    // controls
    r: Vec<glib::WeakRef<gtk::Label>>,
    v: Vec<glib::WeakRef<gtk::Label>>,
    body_selector: glib::WeakRef<gtk::DropDown>,
    method_selector: glib::WeakRef<gtk::DropDown>,
    drawing_area: glib::WeakRef<gtk::DrawingArea>,
    // child process
    subprocess: Option<gio::Subprocess>,
    input: Option<gio::InputStream>,
    cancel_read: Option<gio::Cancellable>,
    line_input: Option<gio::DataInputStream>,
    header_processed: bool,
    suspend: bool,
    // timeout
    source_id: Option<SourceId>,
    // zoom
    zoom_initial: f64,
    zoom: f64
}

impl Context {
    fn new() -> Rc<RefCell<Context>> {
        let r = Rc::new(RefCell::new(Context{
            base: SharedFromThisBase::new(),
            bodies: Vec::new(),
            method: 100,
            active_body: -1,
            r: Vec::new(),
            v: Vec::new(),
            body_selector: glib::WeakRef::new(),
            method_selector: glib::WeakRef::new(),
            drawing_area: glib::WeakRef::new(),
            //
            subprocess: None,
            input: None,
            cancel_read: None,
            line_input: None,
            header_processed: false,
            suspend: false,
            //
            source_id: None,
            //
            zoom_initial: 0.1,
            zoom: 0.1
        }));
        r.borrow_mut().base.initialise(&r);
        r
    }

    fn stop(&mut self) {
        if self.subprocess.is_some() {
            self.cancel_read.as_ref().unwrap().cancel();
            self.subprocess.as_ref().unwrap().force_exit();

            let _ = self.line_input.as_ref().unwrap().close(None::<&gio::Cancellable>);
            let _ = self.input.as_ref().unwrap().close(None::<&gio::Cancellable>);

            self.subprocess = None;
        }
    }

    fn start(&mut self) {
        self.stop();
        self.bodies.clear();
        self.header_processed = false;
        self.suspend = false;
        self.active_body = -1;
        self.spawn();
    }

    fn spawn(&mut self) {
        let mut path = std::env::current_exe().unwrap();
        path.pop();
        path.push("euler");
        let argv = [
            path.as_os_str(),
            OsStr::new("--input"),
            OsStr::new("solar.txt"),
            OsStr::new("--dt"),
            OsStr::new("0.001"),
            OsStr::new("--T"),
            OsStr::new("1e20")
        ];
        let subprocess = gio::Subprocess::newv(&argv, gio::SubprocessFlags::STDOUT_PIPE).expect("cannot start");
        let input = subprocess.stdout_pipe().unwrap();
        let line_input = gio::DataInputStream::new(&input);
        self.subprocess.replace(subprocess);
        self.input.replace(input);
        self.line_input.replace(line_input);
        self.cancel_read.replace(gio::Cancellable::new());
        self.read_child();
    }

    fn read_child(&mut self) {
        let this = self.shared_from_this();
        self.line_input.as_ref().unwrap().read_line_async(
            0.into(),
            self.cancel_read.as_ref(),
            clone!(@strong this => move |x| { this.borrow_mut().on_new_data(x); }) );
    }

    fn on_new_data(&mut self, res: Result<glib::collections::Slice<u8>, glib::Error>) {
        if res.is_err() { return; }

        let unwrapped = res.unwrap();
        let mut parts = unwrapped.split(|x| *x == b' ' || *x == b'\n');
        let first = parts.next().unwrap();
        if first[0] == b't' {
            // skip column names
        } else if first[0] == b'#' {
            let mut body = Body::new();
            // header
            let name = std::str::from_utf8(parts.next().unwrap()).unwrap();
            let m = std::str::from_utf8(parts.next().unwrap()).unwrap().parse::<f64>().unwrap();
            let color = match parts.next() {
                Some(s) => i64::from_str_radix(std::str::from_utf8(s).unwrap(), 16).unwrap(),
                None => 0
            };
            let b = (((color >> 0) & 0xff) as f64) / 256.0;
            let g = (((color >> 8) & 0xff) as f64) / 256.0;
            let r = (((color >> 16) & 0xff) as f64) / 256.0;
            let rad = match parts.next() {
                Some(s) => std::str::from_utf8(s).unwrap().parse::<f64>().unwrap(),
                None => 1.0
            };
            println!("{} {} r{} g{} b{} {}", name, m, r, g, b, rad);
            body.name = String::from(name);
            body.m = m;
            body.cb = b;
            body.cg = g;
            body.cr = r;
            body.rad = rad;
            self.bodies.push(body);
        } else if !self.header_processed {
            self.header_processed = true;
            let model: gtk::StringList = self.body_selector.upgrade().unwrap().model().unwrap().downcast().unwrap();
            for i in 0..self.bodies.len() {
                println!("Append {}", self.bodies[i].name);
                model.append(&self.bodies[i].name);
            }
            self.active_body = 0;
        }

        if self.header_processed {
            // time in first
            for i in 0..self.bodies.len() {
                for j in 0..3 {
                    match parts.next() {
                        Some(s) => {
                            self.bodies[i].r[j] = std::str::from_utf8(s).unwrap().parse::<f64>().unwrap()
                        },
                        _ => break
                    }
                }
                for j in 0..3 {
                    match parts.next() {
                        Some(s) => {
                            self.bodies[i].v[j] = std::str::from_utf8(s).unwrap().parse::<f64>().unwrap()
                        },
                        _ => break
                    }
                }
            }
            self.update_all();
            self.suspend = true;
        }

        if !self.suspend {
            self.read_child();
        }
    }

    fn method_changed(&mut self, selector: &gtk::DropDown) {
        let active = selector.selected();
        if self.method != active {
            self.method = active;
            self.start();
        }
    }

    fn preset_changed(&mut self, selector: &gtk::DropDown) {
        let active = selector.selected();
    }

    fn draw(&mut self, _area: &gtk::DrawingArea, cr: &gtk::cairo::Context, w: i32, h: i32) {
        let bodies = &mut self.bodies;
        let zoom = self.zoom;
        for i in 0..bodies.len() {
            let body = &mut bodies[i];
            let x = body.r[0] * (w as f64) * zoom + (w as f64) / 2.0;
            let y = body.r[1] * (w as f64) * zoom + (h as f64) / 2.0;
            if self.active_body == (i as i32) {
                cr.set_source_rgb(1.0, 0.0, 0.0);
            } else {
                cr.set_source_rgb(body.cr, body.cg, body.cb);
            }
            cr.arc(x, y, 2.0*body.rad, 0.0, 2.0 * std::f64::consts::PI);
            let _ = cr.fill();

            body.x0 = x;
            body.y0 = y;

            if body.show_tip {
                cr.set_font_size(13.0);
                cr.move_to(x, y);
                let _ = cr.show_text(&body.name);
            }
        }
    }

    fn update_all(&mut self) {
        let i = self.active_body;
        if i >= 0 && i < (self.bodies.len() as i32) {
            let body = &self.bodies[i as usize];
            let cx = ['x', 'y', 'z'];
            for j in 0..3 {
                let fmt = format!("<tt>r<sub>{}</sub> = {:+.8e}</tt>", cx[j], body.r[j]);
                self.r[j].upgrade().unwrap().set_label(&fmt);
                let fmt = format!("<tt>v<sub>{}</sub> = {:+.8e}</tt>", cx[j], body.v[j]);
                self.v[j].upgrade().unwrap().set_label(&fmt);
            }
        }
        gtk::Widget::queue_draw(&self.drawing_area.upgrade().unwrap().upcast());
    }

    fn timeout(&mut self) -> glib::ControlFlow {
        if self.header_processed && self.suspend {
            self.suspend = false;
            self.read_child();
        }

        match self.source_id {
            Some(_) => glib::ControlFlow::Continue,
            None => glib::ControlFlow::Break
        }
    }

    fn get_body(&self, x: f64, y: f64) -> i32 {
        let mut mindist = -1.0;
        let mut argmin = -1;
        for i in 0..self.bodies.len()
        {
            let body = &self.bodies[i];
            let dist = (body.x0 - x) * (body.x0 - x) +
                            (body.y0 - y) * (body.y0 - y);
            if argmin < 0 || dist < mindist
            {
                mindist = dist;
                argmin = i as i32;
            }
        }
        if argmin >= 0 && mindist * mindist < 100.0
        {
            argmin
        } else {
            -1
        }
    }

    fn motion_notify(&mut self, _: &gtk::EventControllerMotion, x: f64, y: f64) {
        let argmin = self.get_body(x, y);
        for i in 0..self.bodies.len() {
            self.bodies[i].show_tip = false;
        }
        if argmin >= 0 {
            self.bodies[argmin as usize].show_tip = true;
        }
    }

    fn button_press(&mut self, _: &gtk::GestureClick, _press: i32, x: f64, y: f64) {
        let index = self.get_body(x, y);
        if index >= 0 {
            self.body_selector.upgrade().map(|x| x.set_selected(index as u32));
            self.active_body = index;
        }
    }

    fn zoom_begin(&mut self, _: &gtk::GestureZoom, _: Option<&gtk::gdk::EventSequence>) {
        self.zoom_initial = self.zoom;
    }

    fn zoom_scale_changed(&mut self, _: &gtk::GestureZoom, scale: f64) {
        self.zoom = self.zoom_initial * scale;
        gtk::Widget::queue_draw(&self.drawing_area.upgrade().unwrap().upcast());
    }

    fn mouse_scroll(&mut self, _: &gtk::EventControllerScroll, _dx: f64, dy: f64) -> glib::Propagation {
        if dy > 0.0 {
            self.zoom /= 1.1;
        } else if dy < 0.0 {
            self.zoom *= 1.1;
        }
        gtk::Widget::queue_draw(&self.drawing_area.upgrade().unwrap().upcast());
        glib::Propagation::Stop
    }

    fn close(&mut self) {
        self.source_id.take().map(|source_id| source_id.remove());
        self.stop();
    }
}

fn control_widget(ctx: &Rc<RefCell<Context>>) -> gtk::Widget {
    let frame = gtk::Frame::new(Some("Controls"));
    let bx = gtk::Box::new(gtk::Orientation::Vertical, 0);

    frame.set_child(Some(&bx));
    bx.append(&gtk::Label::new(Some("Preset:")));
    let presets = ["2 Bodies", "3 Bodies", "Solar", "Saturn"];
    let preset_selector = gtk::DropDown::from_strings(&presets);
    preset_selector.connect_state_flags_changed(clone!(@strong ctx => move |a, _| { ctx.borrow_mut().preset_changed(a); } ));
    bx.append(&preset_selector);

    let methods = ["Euler", "Verlet"];
    bx.append(&gtk::Label::new(Some("Method:")));
    let method_selector = gtk::DropDown::from_strings(&methods);
    method_selector.connect_state_flags_changed(clone!(@strong ctx => move |a, _| { ctx.borrow_mut().method_changed(a); } ));
    bx.append(&method_selector);
    bx.append(&gtk::Label::new(Some("Input:")));
    let entry = gtk::Entry::new();
    // TODO signal
    // TODO store buffer
    let buffer = entry.buffer();
    // buffer.set_text() // TODO
    bx.append(&entry);
    bx.append(&gtk::Label::new(Some("dt:")));
    // TODO store dt
    let dt = gtk::SpinButton::with_range(1e-14, 0.1, 0.00001);
    dt.set_digits(8);
    // dt.set_value(); // TODO
    // signal
    bx.append(&dt);

    ctx.borrow_mut().method_selector.set(Some(&method_selector.into()));

    frame.into()
}

fn info_widget(ctx: &Rc<RefCell<Context>>) -> gtk::Widget {
    let frame = gtk::Frame::new(Some("Info"));
    let bx = gtk::Box::new(gtk::Orientation::Vertical, 0);
    frame.set_child(Some(&bx));

    let strings = [];
    let body_selector = gtk::DropDown::from_strings(&strings);
    body_selector.set_selected(0);
    // signal

    bx.append(&body_selector);

    for _i in 0..3 {
        let x = gtk::Label::new(Some("-"));
        bx.append(&x);
        x.set_width_chars(30);
        x.set_use_markup(true);
        ctx.borrow_mut().r.push(gtk::prelude::ObjectExt::downgrade(&x));
    }
    for _i in 0..3 {
        let vx = gtk::Label::new(Some("-"));
        bx.append(&vx);
        vx.set_width_chars(30);
        vx.set_use_markup(true);
        ctx.borrow_mut().v.push(gtk::prelude::ObjectExt::downgrade(&vx));
    }

    ctx.borrow_mut().body_selector.set(Some(&body_selector.into()));

    frame.into()
}

fn right_pane(ctx: &Rc<RefCell<Context>>) -> gtk::Widget {
    let bx = gtk::Box::new(gtk::Orientation::Vertical, 0);

    bx.append(&control_widget(ctx));
    bx.append(&info_widget(ctx));

    bx.set_halign(gtk::Align::End);
    bx.set_valign(gtk::Align::Start);

    bx.into()
}

// When the application is launched…
fn on_activate(application: &gtk::Application, ctx: &Rc<RefCell<Context>>) {
    let window = gtk::ApplicationWindow::new(application);
    window.set_title(Some("N-Body"));
    window.set_default_size(1024, 768);

    let drawing_area = gtk::DrawingArea::new();
    drawing_area.set_vexpand(true);
    drawing_area.set_hexpand(true);
    drawing_area.set_draw_func(clone!(@strong ctx => move |a, b, c, d| ctx.borrow_mut().draw(a, b, c, d)));

    let overlay = gtk::Overlay::new();

    window.set_child(Some(&overlay));

    overlay.set_child(Some(&drawing_area));
    overlay.add_overlay(&right_pane(ctx));

    let motion = gtk::EventControllerMotion::new();
    motion.connect_motion(clone!(@strong ctx => move |a, b, c| ctx.borrow_mut().motion_notify(a, b, c)));
    motion.set_propagation_phase(gtk::PropagationPhase::Capture);
    drawing_area.add_controller(motion.upcast::<gtk::EventController>());

    let gclick = gtk::GestureClick::new();
    gclick.connect_pressed(clone!(@strong ctx => move |a, b, c, d| ctx.borrow_mut().button_press(a, b, c, d)));
    gclick.set_propagation_phase(gtk::PropagationPhase::Capture);
    drawing_area.add_controller(gclick.upcast::<gtk::EventController>());

    let scroll = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
    scroll.connect_scroll(clone!(@strong ctx => move |a, dx, dy| ctx.borrow_mut().mouse_scroll(a, dx, dy)));
    scroll.set_propagation_phase(gtk::PropagationPhase::Capture);
    drawing_area.add_controller(scroll.upcast::<gtk::EventController>());

    let zoom = gtk::GestureZoom::new();
    zoom.connect_begin(clone!(@strong ctx => move |a, b| ctx.borrow_mut().zoom_begin(a, b)));
    zoom.connect_scale_changed(clone!(@strong ctx => move |a, b| ctx.borrow_mut().zoom_scale_changed(a, b)));
    zoom.set_propagation_phase(gtk::PropagationPhase::Capture);
    drawing_area.add_controller(zoom.upcast::<gtk::EventController>());

    // … with a button in it …
    //let button = gtk::Button::with_label("Hello World!");
    // … which closes the window when clicked
    //button.connect_clicked(clone!(@weak window => move |_| window.close()));
    //window.set_child(Some(&button));

    window.connect_destroy(clone!(@strong ctx => move |_| ctx.borrow_mut().close()));

    ctx.borrow_mut().drawing_area.set(Some(&drawing_area));
    ctx.borrow_mut().source_id.replace(glib::timeout_add_local(std::time::Duration::from_millis(16), clone!(@strong ctx => move || ctx.borrow_mut().timeout())));

    window.present();
}

fn main() {
    let ctx = Context::new();

    gtk::disable_setlocale();

    let app = gtk::Application::builder()
        .application_id("gtk.n-bodies")
        .build();
    app.connect_activate(clone!(@weak ctx => move |app| {
        on_activate(&app, &ctx);
    }));
    // Run the application
    app.run();
}
