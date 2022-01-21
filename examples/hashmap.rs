use std::collections::HashMap;
use total_float_wrap::TotalF64;

fn main() {
    let mut triangles: HashMap<TotalF64, Vec<(u32, u32)>> = Default::default();

    let start_adj = 1;
    let end_adj = 10;
    let start_opp = 1;
    let end_opp = 30;

    for adjacent in start_adj..=end_adj {
        for opposite in start_opp..=end_opp {
            triangles
                .entry(f64::atan2(adjacent.into(), opposite.into()).into())
                .or_default()
                .push((adjacent, opposite));
        }
    }

    let (_, vals) = triangles.iter().max_by_key(|v| v.1.len()).unwrap();
    
    println!("For the triangles in the square of points [{start_adj}..{end_adj}]x[{start_opp}..{end_opp}]");
    for (TotalF64(angle), group) in triangles.iter().filter(|v| v.1.len() == vals.len()) {
        println!("The group {group:?} has the maximal members");
        println!(
            "- with an angle of {:.2}° - a ratio of {:.5} between the opposite and the adjacent.",
            angle.to_degrees(), angle.tan()
        );
    }
}
