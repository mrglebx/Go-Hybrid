use hy_check::{check_file, Verdict};
use hy_syntax::parse_file;

fn check_err(src: &str, expected_code: &str) {
    let ast = parse_file("test.hy", src).expect("snippet parses");
    let r = check_file(&ast);
    assert_eq!(r.verdict, Verdict::Reject, "expected reject, got: {:?}", r.diagnostics);
    assert!(r.diagnostics.iter().any(|d| d.code == expected_code), "expected {}, got: {:?}", expected_code, r.diagnostics);
}

fn check_ok(src: &str) {
    let ast = parse_file("test.hy", src).expect("snippet parses");
    let r = check_file(&ast);
    assert_eq!(r.verdict, Verdict::Accept, "expected accept, got: {:?}", r.diagnostics);
}

#[test]
fn test_i01_break_exit_non_owned_origin_rejects_e3004() {
    check_err(
        "fn wm(x map[string]int, done chan int) { done <- 1 }\n         fn spawn2(x map[string]int, done chan int) {\n           cur := map[string]int{}\n           for j in 0..2 { cur = x; if j == 1 { break }; cur = map[string]int{} }\n           task wm(move cur, done)\n         }",
        "HYB-E3004",
    );
}

#[test]
fn test_i02_break_exit_alias_freeze_rejects_e3003() {
    check_err(
        "fn wm(x map[string]int, done chan int) { done <- 1 }\n         fn main() {\n           m := map[string]int{\"a\": 1}; a := map[string]int{}; done := make(chan int, 1)\n           for j in 0..2 { a = m; if j == 1 { break }; a = map[string]int{} }\n           task wm(move m, done); a[\"a\"] = 1; <-done\n         }",
        "HYB-E3003",
    );
}

#[test]
fn test_i03_break_exit_chan_send_move_freeze_rejects_e3003() {
    check_err(
        "fn main() {\n           m := map[string]int{\"a\": 1}; a := map[string]int{}; ch := make(chan map[string]int, 1)\n           for j in 0..2 { a = m; if j == 1 { break }; a = map[string]int{} }\n           ch <- move m; a[\"a\"] = 1\n         }",
        "HYB-E3003",
    );
}

#[test]
fn test_l1_break_exit_const_mutates_param_rejects_e3006() {
    check_err(
        "const tags = map[string]int{\"a\": 1}\n         fn touch(c map[string]int, v int) {\n           t := map[string]int{}\n           for i in 0..2 { t = c; if i >= 0 { break }; t = map[string]int{} }\n           t[\"a\"] = v\n         }\n         fn main() { touch(tags, 1) }",
        "HYB-E3006",
    );
}

#[test]
fn test_l2_break_exit_param_stores_freeze_rejects_e3003() {
    check_err(
        "fn link(dst []map[string]int, src map[string]int, other map[string]int) {\n           tmp := other\n           for i in 0..2 { tmp = src; if i >= 0 { break }; tmp = other }\n           dst[0] = tmp\n         }\n         fn worker(s []map[string]int, done chan int) { done <- 1 }\n         fn main() {\n           m := map[string]int{\"a\": 1}; o := map[string]int{\"b\": 1}; dst := []map[string]int{{}}\n           link(dst, m, o); done := make(chan int, 1); task worker(move dst, done); m[\"a\"] = 2; <-done\n         }",
        "HYB-E3003",
    );
}

#[test]
fn test_closure_break_exit_non_owned_origin_rejects_e3004() {
    check_err(
        "fn wm(x map[string]int, done chan int) { done <- 1 }\n         fn main() {\n           f := fn(x map[string]int, done chan int) {\n             cur := map[string]int{}\n             for j in 0..2 { cur = x; if j == 1 { break }; cur = map[string]int{} }\n             task wm(move cur, done)\n           }\n           f(map[string]int{}, make(chan int, 1))\n         }",
        "HYB-E3004",
    );
}

#[test]
fn test_closure_break_exit_const_write_rejects_e3006() {
    check_err(
        "const tags = map[string]int{\"a\": 1}\n         fn main() {\n           f := fn() {\n             t := map[string]int{}\n             for i in 0..2 { t = tags; if i >= 0 { break }; t = map[string]int{} }\n             t[\"a\"] = 1\n           }\n           f()\n         }",
        "HYB-E3006",
    );
}

#[test]
fn test_sound_loop_break_with_owned_value_stays_accepted() {
    check_ok(
        "fn wm(x map[string]int, done chan int) { done <- 1 }\n         fn main() {\n           cur := map[string]int{\"k\": 1}; done := make(chan int, 1)\n           for i in 0..2 { if i == 1 { break } }\n           task wm(move cur, done); <-done\n         }",
    );
}
