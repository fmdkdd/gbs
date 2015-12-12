extern crate gbs;

use std::env;

fn main() {
  let filename = match env::args().nth(1) {
    Some(s) => s,
    _ => panic!("No GBS file specified")
  };
  let gbs = match gbs::load(filename) {
    Ok(x) => x,
    Err(e) => panic!(e.to_string())
  };

  println!("{:?}", gbs);
  println!("load_addr: {:x}", gbs.load_addr);
  println!("init_addr: {:x}", gbs.init_addr);
  println!("play_addr: {:x}", gbs.play_addr);
  println!("sp: {:x}", gbs.sp);

  // println!("version: {}", gbs.version);
  // println!("n_songs: {}", gbs.n_songs);
  // println!("first_song: {}", gbs.first_song);
  // println!("title: {}", gbs.title);
  // println!("author: {}", gbs.author);
  // println!("copyright: {}", gbs.copyright);
}
