MAKEFLAGS += --warn-undefined-variables
MAKEFLAGS += --no-builtin-rules
MAKEFLAGS += --no-builtin-variables

SHELL := bash
.SHELLFLAGS := -eu -o pipefail -c
.DELETE_ON_ERROR:
.SECONDARY: # don't remove intermediate files
.SECONDEXPANSION:

.PHONY: default
default: play

## Print this message
.PHONY: help
help:
	@printf "Available targets:\n\n"
	@awk '/^[a-zA-Z\-_0-9%:\\]+/ { \
		helpMessage = match(lastLine, /^## (.*)/); \
		if (helpMessage) { \
			helpCommand = $$1; \
			helpMessage = substr(lastLine, RSTART + 3, RLENGTH); \
			gsub("\\\\", "", helpCommand); \
			gsub(":+$$", "", helpCommand); \
			printf "  \x1b[32;01m%-24s\x1b[0m %s\n", helpCommand, helpMessage; \
		} \
	} \
	{ \
		if ($$0 !~ /^.PHONY/) { \
			lastLine = $$0 \
		} \
	} \
	' $(MAKEFILE_LIST)
	@printf "\n"

.PHONY: FORCE
FORCE:

DATA_DIR := data
MODELS_DIR := models

BOARDS_DATA := $(DATA_DIR)/boards.json
NORMALIZATION_PARAMS := $(DATA_DIR)/normalization_params.json

## Play the game (default target)
.PHONY: play
play:
	cargo run --release -- play

## Build the project in release mode
.PHONY: build
build:
	cargo build --release

## Let the computer play the game automatically with aggro-km AI
.PHONY: auto-play-aggro-km
auto-play-aggro-km: $(MODELS_DIR)/ai/aggro-km.json
	cargo run --release -- auto-play $(MODELS_DIR)/ai/aggro-km.json

## Let the computer play the game automatically with defensive-km AI
.PHONY: auto-play-defensive-km
auto-play-defensive-km: $(MODELS_DIR)/ai/defensive-km.json
	cargo run --release -- auto-play $(MODELS_DIR)/ai/defensive-km.json

## Let the computer play the game automatically with aggro-raw AI
.PHONY: auto-play-aggro-raw
auto-play-aggro-raw: $(MODELS_DIR)/ai/aggro-raw.json
	cargo run --release -- auto-play $(MODELS_DIR)/ai/aggro-raw.json

## Let the computer play the game automatically with defensive-raw AI
.PHONY: auto-play-defensive-raw
auto-play-defensive-raw: $(MODELS_DIR)/ai/defensive-raw.json
	cargo run --release -- auto-play $(MODELS_DIR)/ai/defensive-raw.json

## Generate board data JSON file (if missing)
.PHONY: generate-board-data
generate-board-data: $(BOARDS_DATA)

## Regenerate board data JSON file (force rebuild)
.PHONY: regenerate-board-data
regenerate-board-data: REGENERATE_BOARDS_JSON=1
regenerate-board-data: $(BOARDS_DATA)

.PHONY: train-ai-all
train-ai-all: train-ai-aggro-km train-ai-defensive-km train-ai-aggro-raw train-ai-defensive-raw

## Train an aggressive AI using genetic algorithms
.PHONY: train-ai-aggro-km
train-ai-aggro-km: REGENERATE_AI_MODEL_JSON=1
train-ai-aggro-km: $(MODELS_DIR)/ai/aggro-km.json

## Train a defensive AI using genetic algorithms
.PHONY: train-ai-defensive-km
train-ai-defensive-km: REGENERATE_AI_MODEL_JSON=1
train-ai-defensive-km: $(MODELS_DIR)/ai/defensive-km.json

## Train an aggressive AI using genetic algorithms
.PHONY: train-ai-aggro-raw
train-ai-aggro-raw: REGENERATE_AI_MODEL_JSON=1
train-ai-aggro-raw: $(MODELS_DIR)/ai/aggro-raw.json

## Train a defensive AI using genetic algorithms
.PHONY: train-ai-defensive-raw
train-ai-defensive-raw: REGENERATE_AI_MODEL_JSON=1
train-ai-defensive-raw: $(MODELS_DIR)/ai/defensive-raw.json

## Start the TUI for analyzing board features
.PHONY: analyze-board-features
analyze-board-features: $(BOARDS_DATA)
	cargo run --release -- analyze-board-features $(BOARDS_DATA)

## Analyze censoring effects
.PHONY: analyze-censoring
analyze-censoring: $(BOARDS_DATA)
	cargo run --release -- analyze-censoring $(BOARDS_DATA)

## Analyze censoring and export CSV curves
.PHONY: analyze-censoring-km-csv
analyze-censoring-km-csv:
	cargo run --release -- analyze-censoring $(BOARDS_DATA) --km-output-dir $(DATA_DIR)/km_curves

## Generate normalization parameters from KM analysis
.PHONY: generate-normalization
generate-normalization: $(NORMALIZATION_PARAMS)

## Regenerate normalization parameters from KM analysis
.PHONY: regenerate-normalization
regenerate-normalization: REGENERATE_NORMALIZATION_PARAMS=1
regenerate-normalization: $(NORMALIZATION_PARAMS)

## Lint all (check only, no auto-fix)
.PHONY: lint
lint:
	./scripts/lint

## Lint all with auto-fix
.PHONY: lint-fix
lint-fix:
	./scripts/lint --fix

## Lint documentation (check only, no auto-fix)
.PHONY: lint-docs
lint-docs:
	./scripts/lint docs

## Lint documentation with auto-fix
.PHONY: lint-docs-fix
lint-docs-fix:
	./scripts/lint docs --fix

## Lint Rust code (check only, no auto-fix, fast)
.PHONY: lint-rust
lint-rust:
	./scripts/lint rust

## Lint Rust code with auto-fix (slower)
.PHONY: lint-rust-fix
lint-rust-fix:
	./scripts/lint rust --fix

## Lint shell scripts (check only, no auto-fix)
.PHONY: lint-shell
lint-shell:
	./scripts/lint shell

## Lint shell scripts with auto-fix
.PHONY: lint-shell-fix
lint-shell-fix:
	./scripts/lint shell --fix

## Clean build artifacts
.PHONY: clean
clean:
	cargo clean

## Purge all generated data
.PHONY: purge
purge: clean
	rm -rf $(DATA_DIR)

# Artifact generation rules

REGENERATE_BOARDS_JSON=
$(BOARDS_DATA): $$(if $$(REGENERATE_BOARDS_JSON),FORCE) | $(DATA_DIR)/
	cargo run --release -- generate-boards --output $@

REGENERATE_AI_MODEL_JSON=
$(MODELS_DIR)/ai/%.json: $$(if $$(REGENERATE_AI_MODEL_JSON),FORCE) $(BOARDS_DATA) | $(MODELS_DIR)/ai/
	cargo run --release -- train-ai $(BOARDS_DATA) --ai $* --output $@

REGENERATE_NORMALIZATION_PARAMS=
$(NORMALIZATION_PARAMS): $$(if $$(REGENERATE_NORMALIZATION_PARAMS),FORCE) | $(DATA_DIR)/
	cargo run --release -- analyze-censoring $(BOARDS_DATA) --normalization-output $@

# Pattern rule to create directories as needed
%/:
	mkdir -p $@
