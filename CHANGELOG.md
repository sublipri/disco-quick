# Changelog

## 0.2.0 - 2025-05-21

### Fixed
- Panics that occur with recent data dumps

### Added
- Parse release series data (previously missing from the dumps)
- Tests

### Changed
- Parse XML attributes by name instead of relying on their order
- Made some struct fields optional
- Parse empty values as None
- Made ID integer types consistent (u32)
- `DiscogsReader::from_path` takes `AsRef<Path>` instead of `&Path` 
