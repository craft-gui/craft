use std::fs;
use std::path::Path;

use quick_xml::Reader;
use quick_xml::events::Event;

const CLDR_URL: &str =
    "https://raw.githubusercontent.com/unicode-org/cldr/main/common/supplemental/supplementalData.xml";

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();

    let xml = reqwest::blocking::get(CLDR_URL)
        .expect("failed to download CLDR")
        .text()
        .expect("failed to read CLDR response");

    println!("cargo:warning=Downloaded CLDR XML: {} bytes", xml.len());

    println!("cargo:warning=Contains firstDay: {}", xml.contains("<firstDay"));

    fs::write(Path::new(&out_dir).join("supplementalData.xml"), &xml).unwrap();

    let data = generate_week_data(&xml);

    fs::write(Path::new(&out_dir).join("week_data.rs"), data).unwrap();

    println!("cargo:rerun-if-changed=build.rs");
}

fn generate_week_data(xml: &str) -> String {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();

    // Updated to use `pub const`
    let mut output = String::from("pub const FIRST_DAY: &[(&str, &str)] = &[\n");
    let mut count = 0;

    loop {
        match reader.read_event_into(&mut buf) {
            // Match BOTH Start and Empty events
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) if e.name().as_ref() == b"firstDay" => {
                let mut day = None;
                let mut territories = None;
                let mut is_variant = false; // Track if this is an alternate variant

                for attr in e.attributes().flatten() {
                    match attr.key.as_ref() {
                        b"day" => {
                            day = Some(String::from_utf8_lossy(attr.value.as_ref()).into_owned());
                        }
                        b"territories" => {
                            territories = Some(String::from_utf8_lossy(attr.value.as_ref()).into_owned());
                        }
                        b"alt" => {
                            is_variant = true; // Flag this node to be ignored
                        }
                        _ => {}
                    }
                }

                // Only push to output if it is NOT a variant
                if !is_variant && let (Some(day), Some(territories)) = (day, territories) {
                    count += 1;
                    for territory in territories.split_whitespace() {
                        output.push_str(&format!("    (\"{}\", \"{}\"),\n", territory, day));
                    }
                }
            }

            Ok(Event::Eof) => break,

            Err(e) => {
                panic!("XML parse error: {}", e);
            }

            _ => {}
        }
        buf.clear();
    }

    println!("cargo:warning=Found {} firstDay elements (excluding variants)", count);

    output.push_str("];\n");
    output
}
