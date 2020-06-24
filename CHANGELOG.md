# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
* csharp: System must be imported ([#13]).

### Changed
* Parse match blocks better by ignoring end condition for nested groups ([#13]).
* Match arms containing parenthesis are now whitespace sensitive ([#13]).
* Language items are no longer trait objects ([#14]).
* Use a singly-linked list to improve how quickly we can iterate over language items in token streams ([#16]).

### Added
* Patterns are now parsed correctly to support alternatives separated by pipes ([#12]).
* Added `quote_fn!` macro and added `FormatInto` to the prelude ([#13]).

[#16]: https://github.com/udoprog/genco/issues/16
[#14]: https://github.com/udoprog/genco/issues/14
[#13]: https://github.com/udoprog/genco/issues/13
[#12]: https://github.com/udoprog/genco/issues/12

[Unreleased]: https://github.com/udoprog/genco/compare/0.14.2...master