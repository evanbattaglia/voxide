use color_eyre::eyre::Result;
use log::debug;
use regex::{Captures, Replacer};
use std::borrow::Cow;
use crate::transforms_replacement_preprocessor::process_replacement_string;

pub struct TransformsApplicator<'a> {
    transforms: &'a Vec<(String, String)>,
}

pub struct LineNumber(pub usize);

struct LinenoCapturingReplacer<'a> {
    lineno: &'a mut Option<usize>,
    to: &'a str,
}

impl<'a> LinenoCapturingReplacer<'a> {
    pub fn new(lineno: &'a mut Option<usize>, to: &'a str) -> Self {
        Self { lineno, to }
    }
}
impl Replacer for LinenoCapturingReplacer<'_> {
    fn replace_append(&mut self, caps: &Captures<'_>, dst: &mut String) {
        if let Some(lineno_str) = caps.name("lineno") {
            *self.lineno = lineno_str.as_str().parse().ok();
        }
        self.to.replace_append(caps, dst);
    }
}

impl TransformsApplicator<'_> {
    pub fn new(transforms: &Vec<(String, String)>) -> TransformsApplicator {
        TransformsApplicator { transforms }
    }

    pub fn apply_transforms<'a>(
        &self,
        path: &'a str,
    ) -> Result<(Option<LineNumber>, Cow<'a, str>)> {
        let mut result = Cow::Borrowed(path);
        let mut lineno = None;

        debug!("Applying transforms to: {}", path);

        for (from, to) in self.transforms.iter() {
            // check if file exists:
            if !std::path::Path::new(result.as_ref()).exists() {
                // TODO: cache regexes
                let r = regex::Regex::new(from)?;
                let preprocessed_to = process_replacement_string(to);
                let replacer = LinenoCapturingReplacer::new(&mut lineno, preprocessed_to.as_str());
                // TODO: don't copy and re-check existence of file if replace_all has not changed anything
                // for some reason just doing result = r.replace_all(...) isn't working
                result = Cow::Owned(r.replace_all(&result, replacer).to_string());
                if let Some(lineno) = lineno {
                    debug!("Found lineno: {}", lineno);
                }
                debug!("Transformed to: {}", result);
            } else {
                debug!("Found file: {}", result);
                break;
            }
        }
        debug!("End of transforms: {}", result);

        Ok((lineno.map(LineNumber), result))
    }
}
