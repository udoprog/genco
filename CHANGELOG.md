# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
* csharp: System must be imported ([#13]).

### Changed
* Parse match blocks better by ignoring end condition for nested groups ([#13]).

### Added
* Patterns are now parsed correctly to support alternatives separated by pipes ([#12]).
* Added `quote_fn!` macro and added `FormatInto` to the prelude ([#13]).

[#12]: https://github.com/udoprog/genco/issues/12
[#13]: https://github.com/udoprog/genco/issues/13

[Unreleased]: https://github.com/udoprog/genco/compare/0.14.2...master