use std::env;
use std::io::{BufReader, BufRead};
use std::fs::File;

use scanf::sscanf;

struct Body {
    name: String,
    r: [f64; 3],
    v: [f64; 3],
    a: [f64; 3],
    m: f64,
    fixed: bool
}

struct Data {
    bodies: Vec<Body>,
    g: f64,
    dt: f64
}

fn euler_next(data: &mut Data) {
    let n = data.bodies.len();
    let g = data.g;
    let dt = data.dt;

    for i in 0..n {
        let b1 = &data.bodies[i];
        let mut a = [0.0, 0.0, 0.0];
        if b1.fixed { continue; }

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

    for i in 0..3 {
        let b = &mut data.bodies[i];

        for k in 0..3 {
            b.v[k] += dt * b.a[k];
            b.r[k] += dt * b.v[k];
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
        let name = &b.name;
        let m = b.m;
        println!("# {name} {m}");
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
    while t < max_time {
        euler_next(data);
        t += data.dt;
        print(data, t);
    }
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
    for line_wrapped in lines {
        let line = line_wrapped.unwrap();
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
                r : [r0, r1, r2],
                v : [v0, v1, v2],
                a : [0.0, 0.0, 0.0],
                m : m,
                fixed : fixed
            });
        }
    }
}

fn run_test() {

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
