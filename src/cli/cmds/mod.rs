mod actor;
mod target;

pub use actor::*;
pub use target::*;

/// convert attributes passed into what the helper expects
fn form_attributes(attr_args: &[String]) -> Vec<(String, Vec<&str>)> {
    let mut attrs = Vec::new();

    for attr_arg in attr_args {
        if let Some((name, vals)) = attr_arg.split_once(':') {
            let values: Vec<&str> = vals.split(',').collect();
            attrs.push((name.to_string(), values))
        }
    }

    attrs
}
