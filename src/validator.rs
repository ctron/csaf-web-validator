use csaf::csaf::loader::detect_version_with;
use csaf::csaf2_0::loader::load_document as load_2_0;
use csaf::csaf2_1::loader::load_document as load_2_1;
use csaf::validation::{ValidationResult, validate_by_preset};

pub fn validate(json_str: &str, preset: &str) -> Result<ValidationResult, String> {
    let vd =
        detect_version_with(json_str).map_err(|e| format!("Failed to parse document: {e}"))?;

    match vd.version.as_str() {
        "2.0" => {
            let doc =
                load_2_0(vd.data).map_err(|e| format!("Failed to load CSAF 2.0 document: {e}"))?;
            Ok(validate_by_preset(&doc, "2.0", preset))
        }
        "2.1" => {
            let doc =
                load_2_1(vd.data).map_err(|e| format!("Failed to load CSAF 2.1 document: {e}"))?;
            Ok(validate_by_preset(&doc, "2.1", preset))
        }
        other => Err(format!("Unsupported CSAF version: {other}")),
    }
}
