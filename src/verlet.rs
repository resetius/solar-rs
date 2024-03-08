use std::env;
use std::io::{BufReader, BufRead};
use std::fs::File;
use std::process::exit;

use scanf::sscanf;

struct Body {
    name: String,
    color: String,
    rad: f64,
    r: [f64; 3],
    v: [f64; 3],
    a: [f64; 3],
    a_next: [f64; 3],
    m: f64,
    max_rad: f64,
    min_rad: f64,
    fixed: bool
}

struct Data {
    bodies: Vec<Body>,
    g: f64,
    dt: f64
}

fn verlet_init(data: &mut Data) {
    let n = data.bodies.len();
    let g = data.g;

    // new acc
    for i in 0..n {
        let b1 = &data.bodies[i];
        if b1.fixed { continue; }
        let mut a = [0.0, 0.0, 0.0];

        for j in 0..n {
            if i == j { continue; }

            let b2 = &data.bodies[j];

            let mut r = 0.0;
            for k in 0..3 {
                r += (b1.r[k] - b2.r[k]) * (b1.r[k] - b2.r[k]);
            }
            r = f64::sqrt(r);

            for k in 0..3 {
                a[k] += g * b2.m * (b2.r[k] - b1.r[k]) / r / r / r;
            }
        }

        data.bodies[i].a = a;
    }
}

fn verlet_next(data: &mut Data) {
    let n = data.bodies.len();
    let g = data.g;
    let dt = data.dt;

    for i in 0..n {
        let b = &mut data.bodies[i];

        for k in 0..3 {
            // new pos
            b.r[k] = b.r[k] + b.v[k] * dt + b.a[k] * dt * dt * 0.5;
        }

        let mut r = 0.0;
        if b.min_rad >= 0.0 || b.max_rad >= 0.0 {
            for k in 0..3 {
                r += b.r[k] * b.r[k];
            }
            r = f64::sqrt(r);
        }

        if b.min_rad > 0.0 && r < b.min_rad {
            for k in 0..3 {
                b.r[k] = b.min_rad * b.r[k] / r;
            }
        }
        if b.max_rad > 0.0 && r > b.max_rad {
            for k in 0..3 {
                b.r[k] = b.max_rad * b.r[k] / r;
            }
        }
    }

    for i in 0..n {
        let b1 = &data.bodies[i];
        if b1.fixed { continue; }
        let mut a_next = [0.0, 0.0, 0.0];

        for j in 0..n {
            if i == j { continue; }

            let b2 = &data.bodies[j];

            let mut r = 0.0;
            for k in 0..3 {
                r += (b1.r[k] - b2.r[k]) * (b1.r[k] - b2.r[k]);
            }
            r = f64::sqrt(r);

            for k in 0..3 {
                a_next[k] += g * b2.m * (b2.r[k] - b1.r[k]) / r / r / r;
            }
        }

        data.bodies[i].a_next = a_next;
    }

    for i in 0..n {
        let b = &mut data.bodies[i];

        for k in 0..3 {
            // new vel
            b.v[k] = b.v[k] + 0.5 * dt * (b.a[k] + b.a_next[k]);
            // a = new acc
            b.a[k] = b.a_next[k];
        }
    }
}

fn kepler(dt: f64) -> f64 {
    let g = 1.0;
    let mm = 1e5;

    let bodies = vec![
        Body {
            name: String::from("b1"),
            color: String::new(),
            r : [0.0, 0.0, 0.0],
            v : [0.0, 0.0, 0.0],
            a : [0.0, 0.0, 0.0],
            a_next: [0.0, 0.0, 0.0],
            min_rad: 0.0,
            max_rad: 0.0,
            rad: 0.0,
            m : mm,
            fixed : true
        },
        Body {
            name : String::from("b2"),
            color: String::new(),
            r : [0.0, 1.0, 0.0],
            v : [f64::sqrt(g * mm), 0.0, 0.0],
            a : [0.0, 0.0, 0.0],
            a_next: [0.0, 0.0, 0.0],
            min_rad: 0.0,
            max_rad: 0.0,
            rad: 0.0,
            m : 1.0,
            fixed : false
        }
    ];

    let mut data = Data {
        bodies : bodies,
        g : g,
        dt : dt
    };

    let mut max_err = 0.0;
    let max_time = 0.1;
    let mut t = 0.0;

    verlet_init(&mut data);

    while t < max_time {
        verlet_next(&mut data);

        let mut r = 0.0;
        for k in 0..3 {
            r += data.bodies[1].r[k] * data.bodies[1].r[k];
        }
        r = f64::sqrt(r);
        let err = f64::abs(r - 1.0);
        if max_err < err {
            max_err = err;
        }
        t += dt;
    }

    return max_err;
}

fn run_test() {
    let err1 = kepler(0.001);
    let err2 = kepler(0.0001);
    let err3 = kepler(0.00001);
    println!("{err1} {err2} {err3}");
    if err1 / 10.0 < err2 {
        println!("Error1"); exit(1);
    }
    if err1 / 100.0 < err3 {
        println!("Error2"); exit(1);
    }
    println!("Ok");
}

/*
  file format:
  G
  N
  Body1 r0 r1 r2 v0 v1 v2 Mass
  Body2 r0 r1 r2 v0 v1 v2 Mass
  ...
  BodyN r0 r1 r2 v0 v1 v2 Mass
 */

 fn load(data: &mut Data, file_name: &mut String) {
    let file = File::open(file_name).unwrap();
    let buf_reader = BufReader::new(file);
    let mut lines = buf_reader.lines();
    data.g = lines.next().unwrap().unwrap().parse::<f64>().unwrap();
    let nbodies = lines.next().unwrap().unwrap().parse::<usize>().unwrap();
    data.bodies.reserve(nbodies);

    for _ in 0..nbodies {
        let line = lines.next().unwrap().unwrap();
        let mut name = String::new();
        let (mut r0, mut r1, mut r2) = (0.0, 0.0, 0.0);
        let (mut v0, mut v1, mut v2) = (0.0, 0.0, 0.0);
        let mut m = 0.0;
        let fixed = false;

        if sscanf!(&line, "{} {} {} {} {} {} {} {}",
                    name,
                    r0, r1, r2,
                    v0, v1, v2,
                    m).is_ok() {
            data.bodies.push(Body {
                name : name,
                color : String::from("000000"),
                r : [r0, r1, r2],
                v : [v0, v1, v2],
                a : [0.0, 0.0, 0.0],
                a_next : [0.0, 0.0, 0.0],
                min_rad : -1.0,
                max_rad : -1.0,
                rad : -1.0,
                m : m,
                fixed : fixed
            });
        }

        if data.bodies.len() >= nbodies {
            break;
        }
    }

    for line_wrapped in lines {
        let line = line_wrapped.unwrap();

        let mut i = 0;
        let mut color = String::new();
        let mut min_radius = -1.0;
        let mut max_radius = -1.0;
        let mut rad = -1.0;
        if sscanf!(&line, "{} {} {} {} {}", i, color, min_radius, max_radius, rad).is_ok() {
            data.bodies[i].color = color.clone();
            data.bodies[i].min_rad = min_radius;
            data.bodies[i].max_rad = max_radius;
            data.bodies[i].rad = rad;
        }
    }
}


fn print_header(data: &Data) {
    // column names
    print!("t ");
    for i in 0..data.bodies.len() {
        for j in 0..3 {
            print!("r{i},{j} ");
        }
        for j in 0..3 {
            print!("v{i},{j} ");
        }
    }
    println!();
    // comment
    for b in &data.bodies {
        println!("# {} {} {} {}", b.name, b.m, b.color, b.rad);
    }
}

fn print(data: &Data, t: f64) {
    print!("{t} ");
    for b in &data.bodies {
        let (r0, r1, r2) = (b.r[0], b.r[1], b.r[2]);
        let (v0, v1, v2) = (b.v[0], b.v[1], b.v[2]);
        print!("{r0} {r1} {r2} {v0} {v1} {v2} ");
    }
    println!();
}

fn solve(data: &mut Data, max_time: f64) {
    let mut t = 0.0;
    print_header(data);
    print(data, t);
    verlet_init(data);
    while t < max_time {
        verlet_next(data);
        t += data.dt;
        print(data, t);
    }
}

fn usage(cmd: &mut String) {
    eprintln!("{cmd} --input file.txt [--dt 0.001] [--T 10] [--test]");
}

fn main() {
    let mut argv: Vec<String> = env::args().collect();
    let mut file_name = String::new();
    let argc = argv.len();
    let mut i = 1;
    let mut dt = 0.0001;
    let mut max_time = 10.0;
    let mut test_mode = false;

    while i < argc {
        if i < argc-1 && argv[i] == "--input" {
            i += 1;
            file_name = argv[i].clone();
        } else if i < argc-1 && argv[i] == "--dt" {
            i += 1;
            dt = argv[i].parse::<f64>().unwrap();
        } else if i < argc-1 && argv[i] == "--T" {
            i += 1;
            max_time = argv[i].parse::<f64>().unwrap();
        } else if argv[i] == "--test" {
            test_mode = true;
        } else {
            usage(&mut argv[0]); return;
        }
        i += 1;
    }

    if test_mode {
        run_test(); return;
    }

    if file_name.is_empty() {
        usage(&mut argv[0]); return;
    }

    let mut data = Data{
        bodies : Vec::new(),
        g : 1.0,
        dt : dt
    };

    load(&mut data, &mut file_name);
    solve(&mut data, max_time);
}
