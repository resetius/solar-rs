use std::env;

fn solve(_max_time: f64) {

}

fn load(_file_name: &mut String) {

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

    load(&mut file_name);
    solve(max_time);
}
