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
	cargo run --release -- auto-play --ai aggro

## Let the computer play the game automatically with defensive AI
.PHONY: auto-play-defensive
auto-play-defensive:
	cargo run --release -- auto-play --ai defensive

## Generate board data JSON file (if missing)
.PHONY: generate-board-data
generate-board-data: $(DATA_DIR)/boards.json

## Regenerate board data JSON file (force rebuild)
.PHONY: regenerate-board-data
regenerate-board-data: FORCE_REBUILD=1
regenerate-board-data: $(DATA_DIR)/boards.json | $(DATA_DIR)/

## Train an aggressive AI using genetic algorithms
.PHONY: train-ai-aggro
train-ai-aggro:
	cargo run --release -- train-ai --ai aggro

## Train a defensive AI using genetic algorithms
.PHONY: train-ai-defensive
train-ai-defensive:
	cargo run --release -- train-ai --ai defensive

## Start the metric tuning TUI application
.PHONY: tune-metrics
tune-metrics:
	cargo run --release -- tune-metrics $(DATA_DIR)/boards.json

## Clean build artifacts
.PHONY: clean
clean:
	cargo clean

## Purge all generated data
.PHONY: purge
purge: clean
	rm -rf $(DATA_DIR)

# Artifact generation rules

FORCE_REBUILD=

$(DATA_DIR)/boards.json: $$(if $$(FORCE_REBUILD),FORCE) | $(DATA_DIR)/
	cargo run --release -- generate-boards --output $@

# Pattern rule to create directories as needed
%/:
	mkdir -p $@
