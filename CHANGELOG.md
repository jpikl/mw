# 📈 Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2021-03-29

### Added

- `&` filter which splits value using a separator and outputs N-th column.
- `-q, --quote` flag to automatically wrap  output of every pattern expression in quotes.
- `-l, --read-end` flag to require the last input value to be properly terminated.
- `-I, --no-stdin` flag to disable reading values from standard input.

### Changed

- `%` is the default pattern escape character instead of `#`.
- `n` filter (substring) was renamed to `#`.
- `N` filter (substring with backward indexing) was replaced by use of `#` with negative indexing (e.g., `#-2`).
- Parsing of `A+L` range can no longer fail with overflow error. Such range would be now resolved as `A-` (from `A` to end).
- Capture groups of a global regex need to be prefixed with `$` (e.g., `{$1}` instead of `{1}`).
- More lenient number parsing that ignore multiple leading zeros (e.g., `001` is interpreted as `1`).
- Output of `--explain` flag and error output have escaped non-printable and other special characters (newline, tab, etc.).
- Output of `--help-pattern` includes list of escape sequences.
- Output of `--help-filters` flag has more readable layout.
- `-T, --no-trailing-delimiter` flag was renamed to `-L, --no-print-end`.
- `-s, --fail-at-end` flag was renamed to `-F, --fail-at-end`.
- `-b, -diff` flag was renamed to `-d, --diff` flag.

### Fixed

- `A+L` range is correctly evaluated as "from `A` to `A+L`" (not `A+L+1` as previously).
- `-h, --help` flag displays correct position of `--` argument in usage.

## [0.2.0] - 2021-02-14

### Added

- `@` filter (regular expression switch).
- Alternative way to write range of substring filters as `start+length`.

### Changed

- `l` filter (to lowercase) was renamed to `v`.
- `L` filter (to uppercase) was renamed to `^`.
- `0` is now a valid filter a no longer considered error.
- Simplified error message for an invalid range.
- Simplified output of `--help-pattern` and `--help-filters` flags.
- Output of `-h, --help` flag is organized into sections.
- Output of `-h, --help` flag uses more colors in descriptions.
- Regular expression `-e. --regex` / `-E. --regex-filename` is now called *global* instead of *external*.

### Fixed

- `--help-filters` flag displays correct name of `i` / `I` filters.

## [0.1.0] - 2020-12-13

Initial release.

[Unreleased]: https://github.com/jpikl/rew/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/jpikl/rew/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/jpikl/rew/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/jpikl/rew/releases/tag/v0.1.0
