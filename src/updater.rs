pub mod secret_project;

use crate::secret_project::*;

fn main() {
    // let url = "";
    let url = "https://jwade109.github.io/vertex_puzzles/manifest.txt";
    let resp = reqwest::blocking::get(url).expect("request failed");

    if let Ok(text) = resp.text() {
        let lines: Vec<&str> = text.lines().collect();
        println!("Got {} puzzles", lines.len());
        for (id, name) in lines.iter().enumerate() {
            let url = format!(
                "https://jwade109.github.io/vertex_puzzles/{}/puzzle.txt",
                name
            );
            let resp = reqwest::blocking::get(url.clone()).expect("request failed");

            let status = resp.status();

            if let Ok(text) = resp.text() {
                let r: PuzzleFileStorage = serde_yaml::from_str(&text).unwrap();
                println!(
                    "[{}] {}: {} ({}, {}v, {}e)",
                    id,
                    url,
                    status,
                    r.title,
                    r.vertices.len(),
                    r.edges.len()
                );
            }
        }
    }
}
