use std::collections::HashMap;

#[derive(Default, Debug, Clone)]
pub struct Preprocessor {
    macro_symbols: HashMap<String, String>,
}

impl Preprocessor {
    pub fn is_macro_def(line: &str) -> bool {
        line.split_whitespace().next() == Some("@macro")
    }

    pub fn define(&mut self, source: &str) {
        let mut words = source.split_whitespace();
        assert_eq!(words.next().unwrap(), "@macro");
        let ident = words.next().unwrap();
        assert_eq!(words.next().unwrap(), "->");
        let expansion = words.collect::<Vec<&str>>().join(" ");
        self.macro_symbols.insert(ident.into(), expansion);
    }

    pub fn preprocess(&self, source: &mut String) {
        let mut output = String::new();
        for line in source.lines() {
            let mut result = line.to_string();
            for (name, expansion) in &self.macro_symbols {
                result = result.replace(name, expansion);
            }
            output.push_str(&result);
            output.push('\n');
        }
        *source = output;
    }
}
