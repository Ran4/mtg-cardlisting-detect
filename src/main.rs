use regex::Regex;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::File;
use std::io::Read;

#[derive(Debug, Hash, Eq, PartialEq)]
enum CardListingFormat {
    Number,     // e.g. "4 Chandra's spitfire"
    NumberAndX, // e.g. "4x Chandra's Spitfire"
    Plain,      // e.g. Chandra's Spitfire
}

impl CardListingFormat {
    fn regex(&self) -> Regex {
        match self {
            CardListingFormat::NumberAndX => Regex::new(r"^\d+[x|X].+"),
            CardListingFormat::Number => Regex::new(r"^\d+\s+.+"),
            CardListingFormat::Plain => Regex::new(r"^\D"),
        }
        .expect("invalid regex")
    }
}

impl TryFrom<&str> for CardListingFormat {
    type Error = ();

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if s.contains("\n") {
            // If there are "\n"'s in the string, then we got sent multiple lines: detect all
            // CardListingFormat's and pick the most common one, if at least one was detected
            let detected_formats: Vec<CardListingFormat> = s
                .split("\n")
                .map(CardListingFormat::try_from)
                .filter_map(Result::ok) // Ignore any failed detections
                .collect();

            let mut format_counter: HashMap<CardListingFormat, usize> = HashMap::new();

            for detected_format in detected_formats {
                *format_counter.entry(detected_format).or_insert(0) += 1;
            }

            let most_common_format: Option<&CardListingFormat> = format_counter
                .iter()
                .max_by_key(|(_format, count)| count.clone())
                .map(|(format, _count)| format);

            match most_common_format {
                Some(&CardListingFormat::NumberAndX) => Ok(CardListingFormat::NumberAndX),
                Some(&CardListingFormat::Number) => Ok(CardListingFormat::Number),
                Some(&CardListingFormat::Plain) => Ok(CardListingFormat::Plain),
                None => Err(()),
            }
        } else {
            // Just a single line

            let s = s.replace("\t", " ");
            if CardListingFormat::NumberAndX.regex().is_match(&s) {
                Ok(CardListingFormat::NumberAndX)
            } else if CardListingFormat::Number.regex().is_match(&s) {
                Ok(CardListingFormat::Number)
            } else if CardListingFormat::Plain.regex().is_match(&s) {
                Ok(CardListingFormat::Plain)
            } else {
                Err(())
            }
        }
    }
}

fn read_file(filename: &str) -> std::io::Result<String> {
    let mut file = File::open(filename)?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn main() {
    const DEFAULT_FILENAME: &str = "assets/cardlistingformat_number_example_00.txt";

    let args: Vec<String> = std::env::args().collect();
    let filename: &str = args.get(1).map(|s| s.as_ref()).unwrap_or(DEFAULT_FILENAME);

    let filenames = [filename];

    for filename in &filenames {
        let content: String = read_file(filename).expect("Failed loading file");
        println!(
            "Detected `{}` -> {:?}",
            content,
            CardListingFormat::try_from(content.as_ref())
        );
    }
}
