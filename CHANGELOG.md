# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

[Unreleased]: https://github.com/udoprog/genco/compare/0.17.3...master

## [0.17.4]

### Changed
* Update project documentation.

[0.17.4]: https://github.com/udoprog/genco/compare/0.17.3...0.17.4

## [0.17.3]

### Changed
* Fixed badge in project.

[0.17.3]: https://github.com/udoprog/genco/compare/0.17.2...0.17.3

## [0.17.2]

### Added
* Added `Copy` and `Clone` implementations for `FromFn` ([#31]).

### Changed
* Changed internal syntax of doc tests ([#32]).

[#31]: https://github.com/udoprog/genco/issues/31
[#32]: https://github.com/udoprog/genco/issues/32
[0.17.2]: https://github.com/udoprog/genco/compare/0.17.1...0.17.2

## [0.17.1]
### Changed
* Documentation fixes.

[0.17.1]: https://github.com/udoprog/genco/compare/0.17.0...0.17.1
## [0.17.0]

### Added
* Added `FormatInto` implementation for `Arguments<'_>` ([#26]).

### Changed
* All syntax has been changed from using `#` to `$` ([#27]).
* `--cfg genco_nightly` has been deprecated in favor of using byte-span hacks to
  detect whitespace between tokens on the same column.

[#26]: https://github.com/udoprog/genco/issues/26
[#27]: https://github.com/udoprog/genco/issues/27
[0.17.0]: https://github.com/udoprog/genco/compare/0.16.0...0.17.0

## [0.16.0]

### Changed
* Add basic support for using genco to tokenize on stable ([#20]).

## [0.15.1]

### Fixed
* Fixed typos in documentation.
* Fixed new Clippy lints.

## [0.15.0]

### Fixed
* csharp: System must be imported ([#13]).

### Changed
* Parse match blocks better by ignoring end condition for nested groups ([#13]).
* Match arms containing parenthesis are now whitespace sensitive ([#13]).
* Language items are no longer trait objects ([#14]).
* Use a singly-linked list to improve how quickly we can iterate over language items in token streams ([#16]).
* Pass formatting configuration by reference instead of by value when constructing a formatter ([#17]).

### Added
* Patterns are now parsed correctly to support alternatives separated by pipes ([#12]).
* Added `quote_fn!` macro and added `FormatInto` to the prelude ([#13]).

[#17]: https://github.com/udoprog/genco/issues/17
[#16]: https://github.com/udoprog/genco/issues/16
[#14]: https://github.com/udoprog/genco/issues/14
[#13]: https://github.com/udoprog/genco/issues/13
[#12]: https://github.com/udoprog/genco/issues/12
[#20]: https://github.com/udoprog/genco/issues/20

[0.16.0]: https://github.com/udoprog/genco/compare/0.15.0...0.16.0
[0.15.0]: https://github.com/udoprog/genco/compare/0.14.2...0.15.0
[0.15.1]: https://github.com/udoprog/genco/compare/0.15.0...0.15.1
[0.16.0]: https://github.com/udoprog/genco/compare/0.15.1...0.16.0
