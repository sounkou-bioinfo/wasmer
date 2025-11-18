# Makefile for wasmer R package

.PHONY: all build check test doc install clean readme lint

# Default target
all: build

# Build the package
build:
	@echo "Building wasmer R package..."
	@R CMD build .
	
# Check the package
check: build
	@echo "Checking wasmer R package..."
	@R CMD check wasmer_*.tar.gz

# Run tests  
test:
	@echo "Running tests..."
	@Rscript test_runner.R

# Build documentation
doc:
	@echo "Building documentation..."
	@Rscript -e "roxygen2::roxygenise()"

# Compile README
readme:
	@echo "Compiling README..."
	@Rscript -e "if (requireNamespace('rmarkdown', quietly=TRUE)) rmarkdown::render('README.Rmd', output_format='github_document'); if (file.exists('README.md')) cat('✅ README.md generated successfully\\n') else cat('❌ README.md generation failed\\n')"

# Run quick tests to verify everything works
quick-test:
	@echo "Running quick smoke test..."
	@Rscript -e "library(wasmer); result <- wasmer_hello_world_example(); cat('Result:', result, '\\n'); if (grepl('Hello', result, ignore.case=TRUE)) cat('✅ Quick test passed!\\n') else stop('❌ Quick test failed!')"

# Install package locally
install: build
	@echo "Installing wasmer R package..."
	@R CMD INSTALL wasmer_*.tar.gz

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@rm -f wasmer_*.tar.gz
	@rm -rf wasmer.Rcheck
	@rm -f README.md
	@rm -rf target/
	@rm -f src/rust/target/

# Lint the Rust code
lint:
	@echo "Linting Rust code..."
	@cd src/rust && cargo clippy -- -D warnings

# Format the Rust code  
format:
	@echo "Formatting Rust code..."
	@cd src/rust && cargo fmt

# Full development cycle: format, lint, build, test, readme
dev: format lint build test readme
	@echo "Development cycle complete!"

# CI target - runs all checks
ci: lint build test readme
	@echo "CI checks complete!"

# Help target
help:
	@echo "Available targets:"
	@echo "  all        - Build the package (default)"
	@echo "  build      - Build the R package"
	@echo "  check      - Run R CMD check"
	@echo "  test       - Run test suite"
	@echo "  doc        - Generate documentation"
	@echo "  readme     - Compile README.Rmd to README.md"
	@echo "  quick-test - Run a quick smoke test"
	@echo "  install    - Install package locally"
	@echo "  clean      - Clean build artifacts"
	@echo "  lint       - Lint Rust code"
	@echo "  format     - Format Rust code"
	@echo "  dev        - Full development cycle"
	@echo "  ci         - CI checks"
	@echo "  help       - Show this help"
