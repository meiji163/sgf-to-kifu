use sgf_parse::go::Point;
use sgf_parse::{go::Move, go::Prop, SgfNode};
use std::collections::HashMap;

#[derive(Debug)]
pub struct GameMeta {
    pub date: String,
    pub event: String,
    pub black_player: String,
    pub black_rank: String,
    pub white_player: String,
    pub white_rank: String,
    pub komi: String,
    pub result: String,
}

#[derive(Debug, Clone)]
pub struct GameMove {
    pub coords: Point,
    pub is_black: bool,
    pub sequence: usize,
}

pub struct RenderConfig {
    pub board_size: usize,
    pub cell: f64,
    pub margin: f64,
    pub stone_r: f64,
    pub width: f64,
    pub height: f64,
    pub header_h: f64,
    pub footer_h: f64,
}

impl RenderConfig {
    pub fn new() -> RenderConfig {
        let mut c = RenderConfig {
            board_size: 19,
            cell: 30.0,
            margin: 30.0,
            header_h: 70.0,
            footer_h: 120.0,
            stone_r: 0.0,
            width: 0.0,
            height: 0.0,
        };
        c.stone_r = c.cell * 0.44;
        c.width = c.margin * 2.0 + c.cell * (c.board_size - 1) as f64;
        c.height = c.width;

        c
    }
}

// relevant SGF properties
pub fn prop_string(p: Option<&Prop>) -> String {
    if p.is_none() {
        String::new()
    } else {
        match p.unwrap() {
            Prop::DT(t) => t.text.clone(),
            Prop::PB(t) => t.text.clone(),
            Prop::PW(t) => t.text.clone(),
            Prop::EV(t) => t.text.clone(),
            Prop::BR(t) => t.text.clone(),
            Prop::WR(t) => t.text.clone(),
            Prop::RE(t) => t.text.clone(),
            Prop::KM(k) => k.to_string(),
            _ => String::new(),
        }
    }
}

pub fn parse_meta(n: &SgfNode<Prop>) -> GameMeta {
    let mut m = GameMeta {
        date: prop_string(n.get_property("DT")),
        black_player: prop_string(n.get_property("PB")),
        black_rank: prop_string(n.get_property("BR")),
        white_player: prop_string(n.get_property("PW")),
        white_rank: prop_string(n.get_property("WR")),
        event: prop_string(n.get_property("EV")),
        result: prop_string(n.get_property("RE")),
        komi: prop_string(n.get_property("KM")),
    };
    if m.komi == "" {
        m.komi = "0".to_string();
    }
    if m.result == "" {
        m.result = "Unknown".to_string();
    }

    m
}

pub fn get_moves(n: &SgfNode<Prop>) -> Vec<GameMove> {
    let mut stones = Vec::new();
    for (i, node) in n.main_variation().enumerate() {
        for p in node.properties() {
            if let Prop::B(Move::Move(coord)) = p {
                stones.push(GameMove {
                    coords: coord.clone(),
                    is_black: true,
                    sequence: i,
                });
            }
            if let Prop::W(Move::Move(coord)) = p {
                stones.push(GameMove {
                    coords: coord.clone(),
                    is_black: false,
                    sequence: i,
                });
            }
        }
    }
    stones
}

// moves places at the same coordinate
pub fn overlaps(stones: &Vec<GameMove>) -> HashMap<Point, Vec<GameMove>> {
    let mut m = HashMap::new();
    for stone in stones.iter() {
        let found = m.contains_key(&stone.coords);
        if !found {
            m.insert(stone.coords.clone(), vec![stone.clone()]);
        } else {
            m.get_mut(&stone.coords).unwrap().push(stone.clone());
        }
    }
    m.retain(|_, v| v.len() > 1);
    m
}

fn stone_fmt(stone: &GameMove, cx: f64, cy: f64, r: f64, cell: f64) -> String {
    let (fill, stroke) = if stone.is_black {
        ("#111", "#000")
    } else {
        ("#f8f8f8", "#333")
    };

    let mut s = format!(
        r#"<circle cx="{cx}" cy="{cy}" r="{r}" fill="{fill}" stroke="{stroke}" stroke-width="0.8"/>"#
    );

    let text_color = if stone.is_black { "white" } else { "black" };
    let font_size = if stone.sequence >= 100 {
        cell * 0.40
    } else {
        cell * 0.5
    };
    let move_num = stone.sequence;
    s.push_str(&format!(
        r#"<text x="{cx}" y="{cy}" fill="{text_color}" font-size="{font_size}" font-family="sans-serif" text-anchor="middle" dominant-baseline="central">{move_num}</text>"#
    ));

    s
}

fn game_title(m: &GameMeta) -> String {
    let mut s = String::new();
    if m.date != "" {
        s.push_str(&format!("{}: ", m.date));
    }
    s.push_str(&format!(
        "{} {} (B) vs {} {} (W)",
        m.black_player, m.black_rank, m.white_player, m.white_rank
    ));

    s
}

fn overlap_fmt(cfg: &RenderConfig, dups: &Vec<Vec<GameMove>>) -> String {
    let mut svg = String::new();
    let mut cx = cfg.margin;
    let mut cy = cfg.margin;
    let mut line_px = 0.0;
    for dup in dups {
        let dup_px = ((dup.len() + 1) as f64) * cfg.cell;
        // wrap line
        if dup_px + line_px + cfg.margin >= cfg.width {
            cy += cfg.cell;
            cx = cfg.margin;
            line_px = 0.0;
        }
        for s in dup[1..].iter() {
            svg.push_str(stone_fmt(s, cx, cy, cfg.stone_r, cfg.cell).as_str());
            cx += cfg.cell;
        }
        cx -= 0.3 * cfg.cell;
        svg.push_str(&format!(r#"<text x="{cx}" y="{cy}" font-size="16" font-family="sans-serif" text-anchor="middle" dominant-baseline="central">=</text>"#));
        cx += 0.7 * cfg.cell;
        svg.push_str(stone_fmt(dup.first().unwrap(), cx, cy, cfg.stone_r, cfg.cell).as_str());
        cx += 1.5 * cfg.cell;
        line_px += dup_px;
    }
    svg
}

fn board_fmt(cfg: &RenderConfig) -> String {
    let mut svg = String::new();
    for i in 0..cfg.board_size {
        let x = cfg.margin + i as f64 * cfg.cell;
        let y = cfg.margin + i as f64 * cfg.cell;
        let start = cfg.margin;
        let end = cfg.margin + (cfg.board_size - 1) as f64 * cfg.cell;

        let sw = if i == 0 || i == cfg.board_size - 1 {
            2.0
        } else {
            0.8
        };

        svg.push_str(&format!(
            r#"<line x1="{x}" y1="{start}" x2="{x}" y2="{end}" stroke="black" stroke-width="{sw}"/>"#
        ));
        svg.push_str(&format!(
            r#"<line x1="{start}" y1="{y}" x2="{end}" y2="{y}" stroke="black" stroke-width="{sw}"/>"#
        ));
    }

    // star points
    if cfg.board_size == 19 {
        for &(col, row) in &[(3, 3), (15, 3), (3, 15), (15, 15), (9, 9)] {
            let cx = cfg.margin + col as f64 * cfg.cell;
            let cy = cfg.margin + row as f64 * cfg.cell;
            svg.push_str(&format!(
                r#"<circle cx="{cx}" cy="{cy}" r="3" fill="black"/>"#
            ));
        }
    }
    svg
}

pub fn build_svg(meta: &GameMeta, stones: &Vec<GameMove>, cfg: &RenderConfig) -> String {
    let svg_height = cfg.height + cfg.header_h + cfg.footer_h;
    let width = cfg.width;
    let margin = cfg.margin;
    let header_h = cfg.header_h;

    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{svg_height}">"#
    );
    svg.push_str(&format!(
        r##"<rect width="{width}" height="{svg_height}" fill="#ffffff"/>"##
    ));

    // meta
    let title = game_title(meta);
    svg.push_str(&format!(
        r#"<text x="{margin}" y="30" fill="black" font-size="16" font-family="sans-serif">{title}</text>"#
    ));
    let komi = meta.komi.clone();
    svg.push_str(&format!(
        r#"<text x="{margin}" y="50" fill="black" font-size="16" font-family="sans-serif" font-style="italic">Komi: {komi}</text>"#
    ));
    let res = meta.result.clone();
    svg.push_str(&format!(
        r#"<text x="{margin}" y="70" fill="black" font-size="16" font-family="sans-serif" font-style="italic">Result: {res}</text>"#
    ));

    svg.push_str(&format!(r#"<g transform="translate(0, {header_h})">"#));
    svg.push_str(&board_fmt(cfg));

    let dups = overlaps(stones);
    for stone in stones.iter() {
        if let Some(ss) = dups.get(&stone.coords) {
            if ss[0].sequence != stone.sequence {
                continue;
            }
        }
        let col = stone.coords.x;
        let row = stone.coords.y;
        let cx = margin + col as f64 * cfg.cell;
        let cy = margin + row as f64 * cfg.cell;
        svg.push_str(stone_fmt(stone, cx, cy, cfg.stone_r, cfg.cell).as_str());
    }
    svg.push_str("</g>");

    let h2 = header_h + cfg.height;
    svg.push_str(&format!(r#"<g transform="translate(0, {h2})">"#));
    let mut dups_vals: Vec<Vec<GameMove>> = dups.into_values().collect();
    dups_vals.sort_by(|v1, v2| (v1[1].sequence).cmp(&v2[1].sequence));
    svg.push_str(&overlap_fmt(cfg, &dups_vals));
    svg.push_str("</g>");

    svg.push_str("</svg>");
    svg
}
