use serde::{Deserialize, Serialize};
use nyanko::pack::cryptology::Region;
use crate::io::{load_local, save_local};
use std::io::{stdin, stdout, Write};

pub const EXPECTED_HASHES: [(&str, &str); 4] = [
    ("bac299d3cf278544782427ff7c71ef58", "6910fae125547fd957a505c67e1c72bd"), // JP
    ("b9e48b02312e5b3dd60194a03157d70c", "45cad482726268e341f5759230ce8cff"), // EN
    ("264a0ffd5f69d257284b93ae881ce2b6", "213cecb58af008964303ecb2cf0f5373"), // TW
    ("3d22eafdcc4fc2a1379b103970b36217", "4cacdb0839634116caaf0b966638865b"), // KR
];

#[derive(Clone, Serialize, Deserialize, Default, PartialEq, Debug)]
pub struct RegionKey {
    pub key: String,
    pub iv: String,
}

#[derive(Clone, Serialize, Deserialize, Default, PartialEq, Debug)]
pub struct UserKeys {
    #[serde(alias = "jp")]
    pub ja: RegionKey,
    pub en: RegionKey,
    pub tw: RegionKey,
    #[serde(alias = "kr")]
    pub ko: RegionKey,
}

impl UserKeys {
    pub fn load() -> Self {
        if let Some(keys) = load_local("keys.json") {
            return keys;
        }
        if let Some(keys) = load_local("keys") {
            return keys;
        }
        Self::default()
    }

    pub fn save(&self) {
        save_local("keys.json", self);
    }

    pub fn print_status(&self) {
        let validations = self.validate();

        println!("=================================================================================");
        println!("{:<6} | {:<34} | {:<34}", "REGION", "KEY", "IV");
        println!("-------+------------------------------------+------------------------------------");

        let display = |region: &str, key: &str, iv: &str, valid: (bool, bool)| {
            let format_cell = |val: &str, is_valid: bool| -> String {
                let quoted = format!("\"{}\"", val);
                let padded = format!("{:<34}", quoted);
                if is_valid {
                    format!("\x1b[32m{}\x1b[0m", padded)
                } else {
                    format!("\x1b[31m{}\x1b[0m", padded)
                }
            };

            let key_str = format_cell(key, valid.0);
            let iv_str = format_cell(iv, valid.1);

            println!("{:<6} | {} | {}", region, key_str, iv_str);
        };

        display("JP", &self.ja.key, &self.ja.iv, validations[0]);
        display("EN", &self.en.key, &self.en.iv, validations[1]);
        display("TW", &self.tw.key, &self.tw.iv, validations[2]);
        display("KR", &self.ko.key, &self.ko.iv, validations[3]);
        println!("=================================================================================");
    }

    pub fn prompt_interactive_load() -> Self {
        let mut keys = Self::load();
        println!("\n--- BCC Key Configuration Wizard ---");
        println!("Paste your Hex keys and IVs below. Leave blank to skip a field.\n");

        let prompt_field = |label: &str| -> String {
            print!("{}", label);
            if stdout().flush().is_err() { return String::new(); }

            let mut input = String::new();
            if stdin().read_line(&mut input).is_err() { return String::new(); }

            input.retain(|character| !character.is_whitespace());
            input
        };

        let key = prompt_field("Enter JP Key: ");
        if !key.is_empty() { keys.ja.key = key; }
        let iv = prompt_field("Enter JP IV : ");
        if !iv.is_empty() { keys.ja.iv = iv; }

        let key = prompt_field("Enter EN Key: ");
        if !key.is_empty() { keys.en.key = key; }
        let iv = prompt_field("Enter EN IV : ");
        if !iv.is_empty() { keys.en.iv = iv; }

        let key = prompt_field("Enter TW Key: ");
        if !key.is_empty() { keys.tw.key = key; }
        let iv = prompt_field("Enter TW IV : ");
        if !iv.is_empty() { keys.tw.iv = iv; }

        let key = prompt_field("Enter KR Key: ");
        if !key.is_empty() { keys.ko.key = key; }
        let iv = prompt_field("Enter KR IV : ");
        if !iv.is_empty() { keys.ko.iv = iv; }

        keys.save();
        println!("\n✓ Configuration saved to neighboring 'keys.json' file.");
        keys
    }

    pub fn validate(&self) -> [(bool, bool); 4] {
        let check = |val: &str, expected: &str| -> bool {
            let clean_val = val.trim();
            if clean_val.is_empty() { return false; }
            let hash = format!("{:x}", md5::compute(clean_val.as_bytes()));
            hash == expected
        };

        [
            (check(&self.ja.key, EXPECTED_HASHES[0].0), check(&self.ja.iv, EXPECTED_HASHES[0].1)),
            (check(&self.en.key, EXPECTED_HASHES[1].0), check(&self.en.iv, EXPECTED_HASHES[1].1)),
            (check(&self.tw.key, EXPECTED_HASHES[2].0), check(&self.tw.iv, EXPECTED_HASHES[2].1)),
            (check(&self.ko.key, EXPECTED_HASHES[3].0), check(&self.ko.iv, EXPECTED_HASHES[3].1)),
        ]
    }

    pub fn to_nyanko_keys(&self) -> Result<nyanko::pack::cryptology::Keys, String> {
        let mut tuples = Vec::new();

        let mut check_and_push = |region: Region, key: &str, iv: &str| {
            let key_clean = key.trim();
            let iv_clean = iv.trim();
            if key_clean.len() == 32 && iv_clean.len() == 32 {
                tuples.push((region, key_clean.to_string(), iv_clean.to_string()));
            }
        };

        check_and_push(Region::Jp, &self.ja.key, &self.ja.iv);
        check_and_push(Region::En, &self.en.key, &self.en.iv);
        check_and_push(Region::Tw, &self.tw.key, &self.tw.iv);
        check_and_push(Region::Kr, &self.ko.key, &self.ko.iv);

        let ref_tuples: Vec<(Region, &str, &str)> = tuples.iter()
            .map(|(region, key, iv)| (*region, key.as_str(), iv.as_str()))
            .collect();

        nyanko::pack::cryptology::Keys::parse(&ref_tuples).map_err(|error| error.to_string())
    }
}