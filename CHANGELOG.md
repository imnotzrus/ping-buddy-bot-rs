# Changelog

All notable changes to the Ping Buddy Bot project will be documented in this file.

## [Unreleased] - 2024-11-15

### Added
- Custom error types using `thiserror` for better error handling
- Comprehensive input validation for topic names
- Validation module with unit tests
- Extensive documentation throughout the codebase
- Structured logging with different log levels
- Environment variable validation (URLs, token)
- Configuration limits (max topics, max subscribers, etc.)
- Reserved topic names to prevent conflicts
- Authorization checks for callback queries
- Cleanup methods for storage maintenance
- Comprehensive README with usage examples

### Changed
- Refactored storage interface from complex iterators to concrete types (`Vec<String>`)
- Simplified `ChatStorage` API for better maintainability
- Centralized all magic numbers and strings into `constants.rs`
- Improved error messages with context
- Better separation of concerns in handler modules
- Enhanced logging throughout the application
- Updated environment configuration with URL validation

### Fixed
- Memory management issues with string allocations
- Potential memory leaks in storage (added cleanup methods)
- Error handling in message deletion (now logs warnings instead of panicking)
- Proper error propagation throughout the application
- Type safety improvements

### Security
- Added comprehensive input validation
- URL scheme validation for webhooks
- Topic name validation (length, characters, reserved names)
- Rate limiting considerations (max topics/subscribers per chat)

### Performance
- Reduced unnecessary string allocations
- Better use of references where possible
- Optimized storage operations

### Documentation
- Added doc comments to all public APIs
- Documented all modules and functions
- Created comprehensive README
- Added inline code documentation
- Documented error types and their usage

## [0.1.0] - Initial Release

### Added
- Basic topic subscription functionality
- `/list` and `/all` commands
- Topic creation through inline buttons
- User subscription management
- In-memory storage
- Webhook support

