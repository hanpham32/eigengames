use pdf_extract;
use std::fs;
use std::io::Write;

const PATH_TO_FILE: &str = "../../vector_data/nipa_primer.pdf";

fn main() {
    println!("Hello World!");
}

pub fn extract_text_from_pdf(path: &str) -> std::io::Result<()> {
    let bytes = fs::read(path).unwrap();
    let out = pdf_extract::extract_text_from_mem(&bytes).unwrap();
    let mut f = fs::File::create("nipa_primer.txt")?;
    f.write_all(out.as_bytes())?;
    Ok(())
}
