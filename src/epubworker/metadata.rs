use epub_builder::{EpubBuilder, MetadataOpf, ZipLibrary};

use super::BuildError;

pub(super) fn add_optional_metadata(
    builder: &mut EpubBuilder<ZipLibrary>,
    key: &str,
    value: &str,
) -> Result<(), BuildError> {
    if !value.trim().is_empty() {
        builder.metadata(key, value)?;
    }
    Ok(())
}

pub(super) fn add_optional_meta_tag(builder: &mut EpubBuilder<ZipLibrary>, name: &str, value: &str) {
    if value.trim().is_empty() {
        return;
    }
    builder.add_metadata_opf(Box::new(MetadataOpf {
        name: name.to_string(),
        content: value.trim().to_string(),
    }));
}
