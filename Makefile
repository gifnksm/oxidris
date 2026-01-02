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

## Play the game (default target)
.PHONY: play
play:
	cargo run --release -- play

## Build the project in release mode
.PHONY: build
build:
	cargo build --release

## Let the computer play the game automatically with aggressive AI
.PHONY: auto-play-aggro
auto-play-aggro:
	cargo run --release -- auto-play $(MODELS_DIR)/ai/aggro.json

## Let the computer play the game automatically with defensive AI
.PHONY: auto-play-defensive
auto-play-defensive:
	cargo run --release -- auto-play $(MODELS_DIR)/ai/defensive.json

## Generate board data JSON file (if missing)
.PHONY: generate-board-data
generate-board-data: $(DATA_DIR)/boards.json

## Regenerate board data JSON file (force rebuild)
.PHONY: regenerate-board-data
regenerate-board-data: REGENERATE_BOARDS_JSON=1
regenerate-board-data: $(DATA_DIR)/boards.json | $(DATA_DIR)/

.PHONY: regenerate-board-feature-stats
regenerate-board-feature-stats: $(DATA_DIR)/boards.json
	cargo run --release -- generate-board-feature-stats \
		$(DATA_DIR)/boards.json \
		--output crates/oxidris-ai/src/board_feature/stats.rs

## Train an aggressive AI using genetic algorithms
.PHONY: train-ai-aggro
train-ai-aggro:
	cargo run --release -- train-ai --ai aggro --output $(MODELS_DIR)/ai/aggro.json

## Train a defensive AI using genetic algorithms
.PHONY: train-ai-defensive
train-ai-defensive:
	cargo run --release -- train-ai --ai defensive --output $(MODELS_DIR)/ai/defensive.json

## Start the TUI for analyzing board features
.PHONY: analyze-board-features
analyze-board-features:
	cargo run --release -- analyze-board-features $(DATA_DIR)/boards.json

## Analyze analyzing censoring effects
.PHONY: analyze-censoring
analyze-censoring:
	cargo run --release -- analyze-censoring $(DATA_DIR)/boards.json

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

$(DATA_DIR)/boards.json: $$(if $$(REGENERATE_BOARDS_JSON),FORCE) | $(DATA_DIR)/
	cargo run --release -- generate-boards --output $@

# Pattern rule to create directories as needed
%/:
	mkdir -p $@
