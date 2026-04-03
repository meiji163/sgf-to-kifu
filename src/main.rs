use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::process;

use exitcode;
use resvg::tiny_skia::{self, Pixmap};
use resvg::usvg;
use sgf_parse::parse;

mod kifu;
use kifu::*;

fn rasterize(svg: String, cfg: &RenderConfig) -> Pixmap {
    let mut opts = usvg::Options::default();
    opts.fontdb_mut().load_system_fonts();
    let tree = usvg::Tree::from_str(&svg, &opts).unwrap();

    let scale = 2.0; // 2x for retina-quality output
    let pix_w = (cfg.width * scale) as u32;
    let pix_h = ((cfg.height + cfg.header_h + cfg.footer_h) * scale) as u32;
    let mut pixmap = Pixmap::new(pix_w, pix_h).unwrap();

    let transform = tiny_skia::Transform::from_scale(scale as f32, scale as f32);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    pixmap
}

fn run(file_name: &String, cfg: &RenderConfig) -> io::Result<()> {
    let sgf = fs::read_to_string(file_name)?;
    let collection = parse(&sgf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let root_node = collection.first().unwrap();
    let go_node = root_node
        .as_go_node()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let meta = parse_meta(go_node);
    println!("Meta: {:?}", meta);

    let stones = get_moves(go_node);

    let svg = build_svg(&meta, &stones, &cfg);

    let svg_path = Path::new(&file_name).with_extension("svg");
    fs::write(svg_path.to_string_lossy().to_string(), &svg)?;

    let png_path = Path::new(&file_name).with_extension("png");
    let pixmap = rasterize(svg, &cfg);
    pixmap.save_png(&png_path)?;

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <sgf-file> [<sgf-file-2>...]", args[0]);
        process::exit(exitcode::DATAERR);
    }

    let cfg = RenderConfig::new();
    for file_name in args[1..].iter() {
        if let Err(e) = run(file_name, &cfg) {
            eprintln!("Error: {}", e);
            process::exit(exitcode::DATAERR);
        }
    }
    process::exit(exitcode::OK);
}
